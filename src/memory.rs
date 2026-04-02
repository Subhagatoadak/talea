// src/memory.rs
//
// Memory management for Talea.
//
// Provides:
//  - `MemoryTracker`: a lightweight accounting layer that records how many
//    bytes are being "held" by the Talea runtime and enforces a configurable
//    ceiling.  All sizes are *approximate* — Rust values on the heap are
//    measured by a recursive `byte_size()` helper rather than by hooking the
//    global allocator, keeping the implementation dependency-free.
//
//  - `TaleaArena`: a bump-allocation arena for short-lived string data.
//    Built on top of `bumpalo`, it lets heavy text-processing passes allocate
//    thousands of small strings from a single contiguous block that is freed
//    in one shot when the arena is dropped (e.g., at the end of a
//    `with memory scope do … end` block).
//
//  - `byte_size()`: a recursive helper that estimates the heap footprint of a
//    `TaleaValue`.

use bumpalo::Bump;
use std::collections::HashMap;

use crate::runtime::TaleaValue;

// ─────────────────────────────────────────────────────────────────────────────
// Byte-size estimator
// ─────────────────────────────────────────────────────────────────────────────

/// Estimate the heap footprint of a `TaleaValue` in bytes.
///
/// String capacity is used (not length) because Rust keeps the full allocated
/// buffer even when only part of it is used.  Collections include their
/// own heap allocations recursively.
pub fn byte_size(value: &TaleaValue) -> usize {
    match value {
        TaleaValue::String(s)  => s.capacity(),
        TaleaValue::Number(_)  => 8,
        TaleaValue::Float(_)   => 8,
        TaleaValue::Bool(_)    => 1,
        TaleaValue::Null       => 0,
        TaleaValue::Error(s)   => s.capacity(),
        TaleaValue::Unit(_)    => 8,
        TaleaValue::Channel(_) => 64,  // channel id string + queue overhead
        TaleaValue::List(arc) => {
            // The Arc + RwLock themselves live on the stack; the Vec is heap.
            if let Ok(guard) = arc.try_read() {
                let vec_overhead = guard.capacity() * std::mem::size_of::<TaleaValue>();
                let contents: usize = guard.iter().map(byte_size).sum();
                vec_overhead + contents
            } else {
                0
            }
        }
        TaleaValue::Tuple(t) => {
            let vec_overhead = t.capacity() * std::mem::size_of::<TaleaValue>();
            let contents: usize = t.iter().map(byte_size).sum();
            vec_overhead + contents
        }
        TaleaValue::Table(rows) => {
            rows.iter().map(|row| {
                row.iter().map(|(k, v)| k.capacity() + byte_size(v)).sum::<usize>()
            }).sum()
        }
        TaleaValue::Map(m) => {
            m.iter().map(|(k, v)| k.capacity() + byte_size(v)).sum()
        }
        TaleaValue::DbResult { rows, columns, .. } => {
            let col_bytes: usize = columns.iter().map(|c| c.capacity()).sum();
            let row_bytes: usize = rows.iter().map(|row| {
                row.iter().map(|(k, v)| k.capacity() + byte_size(v)).sum::<usize>()
            }).sum();
            col_bytes + row_bytes
        }
        TaleaValue::PyObject(_) => 128,  // opaque; use a conservative estimate
        #[cfg(feature = "r")]
        TaleaValue::RObject(_) => 128,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MemoryTracker
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks approximate heap usage and enforces an optional ceiling.
///
/// The interpreter calls `track_define` whenever a new variable is assigned
/// and `track_free` when `free VAR` is called.  This allows accurate
/// bookkeeping even though we don't hook the system allocator.
pub struct MemoryTracker {
    /// Bytes currently accounted for across all live variables.
    used_bytes: usize,
    /// Maximum allowed bytes (0 = unlimited).
    limit_bytes: usize,
    /// Per-variable byte counts so we can subtract precisely on `free`.
    variable_sizes: HashMap<String, usize>,
}

impl MemoryTracker {
    /// Create a tracker with no limit.
    pub fn new() -> Self {
        MemoryTracker {
            used_bytes: 0,
            limit_bytes: 0,
            variable_sizes: HashMap::new(),
        }
    }

    /// Set a memory ceiling in megabytes.  Pass `0` to disable the limit.
    pub fn set_limit_mb(&mut self, mb: usize) {
        self.limit_bytes = mb * 1_048_576;
    }

    /// Record a variable being defined or redefined.
    ///
    /// Returns `Err` if the new allocation would breach the limit.
    pub fn track_define(&mut self, name: &str, value: &TaleaValue) -> Result<(), String> {
        let old_size = self.variable_sizes.get(name).copied().unwrap_or(0);
        let new_size = byte_size(value);
        let delta = new_size.saturating_sub(old_size);

        if self.limit_bytes > 0 && self.used_bytes + delta > self.limit_bytes {
            return Err(format!(
                "Memory limit exceeded: tried to allocate {} KB but only {} KB remain (limit: {} MB).",
                delta / 1024,
                (self.limit_bytes - self.used_bytes) / 1024,
                self.limit_bytes / 1_048_576,
            ));
        }

        self.used_bytes = self.used_bytes.saturating_sub(old_size) + new_size;
        self.variable_sizes.insert(name.to_string(), new_size);
        Ok(())
    }

    /// Record a variable being freed.
    pub fn track_free(&mut self, name: &str) {
        if let Some(size) = self.variable_sizes.remove(name) {
            self.used_bytes = self.used_bytes.saturating_sub(size);
        }
    }

    /// Human-readable usage summary.
    pub fn usage_summary(&self) -> String {
        let used_kb = self.used_bytes as f64 / 1024.0;
        let used_mb = used_kb / 1024.0;
        if self.limit_bytes == 0 {
            format!("Memory usage: {:.1} MB ({:.0} KB) — no limit set", used_mb, used_kb)
        } else {
            let limit_mb = self.limit_bytes as f64 / 1_048_576.0;
            let pct = (self.used_bytes as f64 / self.limit_bytes as f64) * 100.0;
            format!(
                "Memory usage: {:.1} MB / {:.0} MB ({:.1}% of limit)",
                used_mb, limit_mb, pct
            )
        }
    }

    /// Raw bytes used.
    pub fn bytes_used(&self) -> usize {
        self.used_bytes
    }

    /// Returns `true` if usage is within the limit (or no limit is set).
    pub fn is_within_limit(&self) -> bool {
        self.limit_bytes == 0 || self.used_bytes <= self.limit_bytes
    }

    /// Snapshot: returns a sorted list of (variable_name, bytes) pairs.
    pub fn top_consumers(&self, n: usize) -> Vec<(String, usize)> {
        let mut pairs: Vec<(String, usize)> = self.variable_sizes
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TaleaArena
// ─────────────────────────────────────────────────────────────────────────────

/// A bump-allocation arena for short-lived string allocations.
///
/// Use this inside `with memory scope do … end` blocks: allocate many strings
/// cheaply, then drop the entire arena at block exit — O(1) regardless of how
/// many strings were allocated.
///
/// # Example (interpreter internals)
/// ```ignore
/// let mut arena = TaleaArena::new(64);   // 64 MB limit
/// let s = arena.alloc_str("hello world");
/// // … lots of processing …
/// drop(arena);  // frees everything at once
/// ```
pub struct TaleaArena {
    bump: Bump,
    limit_bytes: usize,
}

impl TaleaArena {
    /// Create a new arena with the given limit in megabytes.
    /// Pass `0` to disable the limit.
    pub fn new(limit_mb: usize) -> Self {
        TaleaArena {
            bump: Bump::new(),
            limit_bytes: limit_mb * 1_048_576,
        }
    }

    /// Allocate a string copy inside the arena.
    ///
    /// Returns `Err` if the allocation would exceed the configured limit.
    pub fn alloc_str<'a>(&'a self, s: &str) -> Result<&'a str, String> {
        if self.limit_bytes > 0 && self.bump.allocated_bytes() + s.len() > self.limit_bytes {
            return Err(format!(
                "Arena memory limit ({} MB) exceeded while trying to allocate {} bytes.",
                self.limit_bytes / 1_048_576,
                s.len(),
            ));
        }
        Ok(self.bump.alloc_str(s))
    }

    /// Total bytes allocated from this arena so far.
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Human-readable allocation summary.
    pub fn summary(&self) -> String {
        let kb = self.bump.allocated_bytes() as f64 / 1024.0;
        if self.limit_bytes == 0 {
            format!("Arena: {:.1} KB allocated (no limit)", kb)
        } else {
            let limit_mb = self.limit_bytes as f64 / 1_048_576.0;
            format!("Arena: {:.1} KB / {:.0} MB limit", kb, limit_mb)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_respects_limit() {
        let mut tracker = MemoryTracker::new();
        tracker.set_limit_mb(1); // 1 MB

        // A small string should succeed
        let small = TaleaValue::String("hello".to_string());
        assert!(tracker.track_define("x", &small).is_ok());

        // A 2 MB string should fail
        let big = TaleaValue::String("x".repeat(2 * 1_048_576));
        assert!(tracker.track_define("y", &big).is_err());
    }

    #[test]
    fn tracker_free_reduces_usage() {
        let mut tracker = MemoryTracker::new();
        let val = TaleaValue::String("hello world".to_string());
        tracker.track_define("a", &val).unwrap();
        let before = tracker.bytes_used();
        tracker.track_free("a");
        assert!(tracker.bytes_used() < before);
        assert_eq!(tracker.bytes_used(), 0);
    }

    #[test]
    fn arena_alloc_str_works() {
        let arena = TaleaArena::new(0); // no limit
        let s = arena.alloc_str("test string").unwrap();
        assert_eq!(s, "test string");
        assert!(arena.allocated_bytes() >= 11);
    }

    #[test]
    fn arena_respects_limit() {
        let arena = TaleaArena::new(1); // 1 MB
        let big = "x".repeat(2 * 1_048_576);
        assert!(arena.alloc_str(&big).is_err());
    }
}
