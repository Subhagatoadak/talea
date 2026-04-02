// src/db.rs
//
// Database abstraction layer for Talea.
//
// Wraps sqlx connection pools behind an enum so the interpreter never has to
// know which concrete database it is talking to.  All blocking operations are
// run via `tokio::runtime::Runtime::block_on` so the synchronous interpreter
// can call async sqlx without restructuring the whole runtime.
//
// Supported backends (enabled via the `db` feature flag):
//   - PostgreSQL  (`connect to postgres "postgresql://…" as db`)
//   - MySQL       (`connect to mysql "mysql://…" as db`)
//   - SQLite      (`connect to sqlite "file:mydb.db" as db`)
//
// Without the `db` feature, every public function returns a descriptive error
// asking the user to rebuild with `--features db`.

use std::collections::HashMap;
use crate::runtime::TaleaValue;

// ─────────────────────────────────────────────────────────────────────────────
// DbPool — the connection pool enum
// ─────────────────────────────────────────────────────────────────────────────

/// A live database connection pool.
///
/// Each variant wraps an sqlx pool for the corresponding database engine.
/// The pool is `Clone`-able (it is internally reference-counted by sqlx) so
/// it can be stored as a `TaleaValue` indirection via the interpreter's
/// `db_connections` registry.
#[cfg(feature = "db")]
pub enum DbPool {
    Postgres(sqlx::PgPool),
    Mysql(sqlx::MySqlPool),
    Sqlite(sqlx::SqlitePool),
}

/// Placeholder used when the `db` feature is disabled at compile time.
#[cfg(not(feature = "db"))]
pub enum DbPool {}

// ─────────────────────────────────────────────────────────────────────────────
// ConnectionRegistry — owned by the Interpreter
// ─────────────────────────────────────────────────────────────────────────────

/// Maps alias names (e.g. `"db"`, `"my_conn"`) to live pools.
pub struct ConnectionRegistry {
    pools: HashMap<String, DbPool>,
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        ConnectionRegistry { pools: HashMap::new() }
    }

    pub fn insert(&mut self, alias: String, pool: DbPool) {
        self.pools.insert(alias, pool);
    }

    pub fn get(&self, alias: &str) -> Option<&DbPool> {
        self.pools.get(alias)
    }

    pub fn remove(&mut self, alias: &str) -> bool {
        self.pools.remove(alias).is_some()
    }

    pub fn contains(&self, alias: &str) -> bool {
        self.pools.contains_key(alias)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public helpers called by the interpreter
// ─────────────────────────────────────────────────────────────────────────────

/// Open a connection pool and register it.
///
/// `db_type` should be `"postgres"`, `"mysql"`, or `"sqlite"`.
/// `url` is the connection string or a reference resolved by the interpreter.
pub fn connect(
    registry: &mut ConnectionRegistry,
    alias: &str,
    db_type: &str,
    url: &str,
) -> Result<(), String> {
    #[cfg(feature = "db")]
    {
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let pool = rt.block_on(async {
            match db_type.to_lowercase().as_str() {
                "postgres" | "postgresql" => {
                    sqlx::postgres::PgPoolOptions::new()
                        .max_connections(5)
                        .connect(url)
                        .await
                        .map(DbPool::Postgres)
                        .map_err(|e| e.to_string())
                }
                "mysql" => {
                    sqlx::mysql::MySqlPoolOptions::new()
                        .max_connections(5)
                        .connect(url)
                        .await
                        .map(DbPool::Mysql)
                        .map_err(|e| e.to_string())
                }
                "sqlite" => {
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(5)
                        .connect(url)
                        .await
                        .map(DbPool::Sqlite)
                        .map_err(|e| e.to_string())
                }
                other => Err(format!("Unknown database type '{}'. Use postgres, mysql, or sqlite.", other)),
            }
        })?;
        registry.insert(alias.to_string(), pool);
        println!("[DB: Connected to {} as '{}']", db_type, alias);
        Ok(())
    }

    #[cfg(not(feature = "db"))]
    {
        let _ = (registry, alias, db_type, url);
        Err("Database support is not enabled. Rebuild with `--features db`.".to_string())
    }
}

/// Execute a SELECT query and return the rows as a `TaleaValue::DbResult`.
///
/// `params` may be empty for parameter-free queries.
pub fn query(
    registry: &ConnectionRegistry,
    alias: &str,
    sql: &str,
    params: &[TaleaValue],
) -> Result<TaleaValue, String> {
    #[cfg(feature = "db")]
    {
        let pool = registry.get(alias)
            .ok_or_else(|| format!("No database connection '{}'. Did you run `connect to …`?", alias))?;

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        rt.block_on(async {
            match pool {
                DbPool::Postgres(pg) => query_postgres(pg, sql, params).await,
                DbPool::Mysql(my)    => query_mysql(my, sql, params).await,
                DbPool::Sqlite(sq)   => query_sqlite(sq, sql, params).await,
            }
        })
    }

    #[cfg(not(feature = "db"))]
    {
        let _ = (registry, alias, sql, params);
        Err("Database support is not enabled. Rebuild with `--features db`.".to_string())
    }
}

/// Execute a non-SELECT statement (INSERT, UPDATE, DELETE, DDL).
pub fn execute(
    registry: &ConnectionRegistry,
    alias: &str,
    sql: &str,
    params: &[TaleaValue],
) -> Result<u64, String> {
    #[cfg(feature = "db")]
    {
        let pool = registry.get(alias)
            .ok_or_else(|| format!("No database connection '{}'.", alias))?;

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        rt.block_on(async {
            match pool {
                DbPool::Postgres(pg) => execute_postgres(pg, sql, params).await,
                DbPool::Mysql(my)    => execute_mysql(my, sql, params).await,
                DbPool::Sqlite(sq)   => execute_sqlite(sq, sql, params).await,
            }
        })
    }

    #[cfg(not(feature = "db"))]
    {
        let _ = (registry, alias, sql, params);
        Err("Database support is not enabled. Rebuild with `--features db`.".to_string())
    }
}

/// Insert rows from a `TaleaValue::Table` or `TaleaValue::List` into a table.
pub fn insert(
    registry: &ConnectionRegistry,
    alias: &str,
    table_name: &str,
    data: &TaleaValue,
) -> Result<u64, String> {
    #[cfg(feature = "db")]
    {
        let rows = match data {
            TaleaValue::Table(rows) => rows.clone(),
            TaleaValue::DbResult { rows, .. } => rows.clone(),
            _ => return Err("insert expects a Table or DbResult value".to_string()),
        };

        if rows.is_empty() {
            return Ok(0);
        }

        // Build INSERT … VALUES … statement from the first row's keys
        let columns: Vec<String> = rows[0].keys().cloned().collect();
        let col_list = columns.join(", ");
        let placeholders = columns.iter().enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>().join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, col_list, placeholders);

        let pool = registry.get(alias)
            .ok_or_else(|| format!("No database connection '{}'.", alias))?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

        let mut total_rows: u64 = 0;
        for row in &rows {
            let params: Vec<TaleaValue> = columns.iter()
                .map(|col| row.get(col).cloned().unwrap_or(TaleaValue::Null))
                .collect();
            let affected = rt.block_on(async {
                match pool {
                    DbPool::Postgres(pg) => execute_postgres(pg, &sql, &params).await,
                    DbPool::Mysql(my)    => execute_mysql(my, &sql, &params).await,
                    DbPool::Sqlite(sq)   => execute_sqlite(sq, &sql, &params).await,
                }
            })?;
            total_rows += affected;
        }
        Ok(total_rows)
    }

    #[cfg(not(feature = "db"))]
    {
        let _ = (registry, alias, table_name, data);
        Err("Database support is not enabled. Rebuild with `--features db`.".to_string())
    }
}

/// Close and remove a connection pool.
pub fn close(registry: &mut ConnectionRegistry, alias: &str) -> Result<(), String> {
    if registry.remove(alias) {
        println!("[DB: Closed connection '{}']", alias);
        Ok(())
    } else {
        Err(format!("No connection '{}' to close.", alias))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal async helpers (only compiled with `db` feature)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "db")]
async fn query_postgres(
    pool: &sqlx::PgPool,
    sql: &str,
    params: &[TaleaValue],
) -> Result<TaleaValue, String> {
    use sqlx::Row;
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_postgres(q, p);
    }
    let rows = q.fetch_all(pool).await.map_err(|e| format!("Postgres query error: {}", e))?;

    if rows.is_empty() {
        return Ok(TaleaValue::DbResult { rows: vec![], columns: vec![], count: 0 });
    }

    let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
    let result_rows: Vec<HashMap<String, TaleaValue>> = rows.iter().map(|row| {
        let mut map = HashMap::new();
        for col in row.columns() {
            let name = col.name().to_string();
            let val = pg_row_to_talea(row, col.ordinal());
            map.insert(name, val);
        }
        map
    }).collect();

    let count = result_rows.len();
    Ok(TaleaValue::DbResult { rows: result_rows, columns, count })
}

#[cfg(feature = "db")]
fn bind_postgres<'q>(
    q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    val: &TaleaValue,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match val {
        TaleaValue::String(s)  => q.bind(s.clone()),
        TaleaValue::Number(n)  => q.bind(*n),
        TaleaValue::Float(f)   => q.bind(*f),
        TaleaValue::Bool(b)    => q.bind(*b),
        TaleaValue::Null       => q.bind(Option::<String>::None),
        _ => q.bind(format!("{}", val)),
    }
}

#[cfg(feature = "db")]
fn pg_row_to_talea(row: &sqlx::postgres::PgRow, idx: usize) -> TaleaValue {
    use sqlx::Row;
    // Try common types in order of specificity
    if let Ok(v) = row.try_get::<i64, _>(idx) {
        return TaleaValue::Number(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(idx) {
        return TaleaValue::Float(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(idx) {
        return TaleaValue::Bool(v);
    }
    if let Ok(v) = row.try_get::<String, _>(idx) {
        return TaleaValue::String(v);
    }
    TaleaValue::Null
}

#[cfg(feature = "db")]
async fn execute_postgres(
    pool: &sqlx::PgPool,
    sql: &str,
    params: &[TaleaValue],
) -> Result<u64, String> {
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_postgres(q, p);
    }
    let result = q.execute(pool).await.map_err(|e| format!("Postgres execute error: {}", e))?;
    Ok(result.rows_affected())
}

#[cfg(feature = "db")]
async fn query_mysql(
    pool: &sqlx::MySqlPool,
    sql: &str,
    params: &[TaleaValue],
) -> Result<TaleaValue, String> {
    use sqlx::Row;
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_mysql(q, p);
    }
    let rows = q.fetch_all(pool).await.map_err(|e| format!("MySQL query error: {}", e))?;

    if rows.is_empty() {
        return Ok(TaleaValue::DbResult { rows: vec![], columns: vec![], count: 0 });
    }

    let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
    let result_rows: Vec<HashMap<String, TaleaValue>> = rows.iter().map(|row| {
        let mut map = HashMap::new();
        for col in row.columns() {
            let name = col.name().to_string();
            let val = mysql_row_to_talea(row, col.ordinal());
            map.insert(name, val);
        }
        map
    }).collect();

    let count = result_rows.len();
    Ok(TaleaValue::DbResult { rows: result_rows, columns, count })
}

#[cfg(feature = "db")]
fn bind_mysql<'q>(
    q: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    val: &TaleaValue,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match val {
        TaleaValue::String(s)  => q.bind(s.clone()),
        TaleaValue::Number(n)  => q.bind(*n),
        TaleaValue::Float(f)   => q.bind(*f),
        TaleaValue::Bool(b)    => q.bind(*b),
        TaleaValue::Null       => q.bind(Option::<String>::None),
        _ => q.bind(format!("{}", val)),
    }
}

#[cfg(feature = "db")]
fn mysql_row_to_talea(row: &sqlx::mysql::MySqlRow, idx: usize) -> TaleaValue {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<i64, _>(idx) { return TaleaValue::Number(v); }
    if let Ok(v) = row.try_get::<f64, _>(idx) { return TaleaValue::Float(v); }
    if let Ok(v) = row.try_get::<bool, _>(idx) { return TaleaValue::Bool(v); }
    if let Ok(v) = row.try_get::<String, _>(idx) { return TaleaValue::String(v); }
    TaleaValue::Null
}

#[cfg(feature = "db")]
async fn execute_mysql(
    pool: &sqlx::MySqlPool,
    sql: &str,
    params: &[TaleaValue],
) -> Result<u64, String> {
    let mut q = sqlx::query(sql);
    for p in params { q = bind_mysql(q, p); }
    let result = q.execute(pool).await.map_err(|e| format!("MySQL execute error: {}", e))?;
    Ok(result.rows_affected())
}

#[cfg(feature = "db")]
async fn query_sqlite(
    pool: &sqlx::SqlitePool,
    sql: &str,
    params: &[TaleaValue],
) -> Result<TaleaValue, String> {
    use sqlx::Row;
    let mut q = sqlx::query(sql);
    for p in params { q = bind_sqlite(q, p); }
    let rows = q.fetch_all(pool).await.map_err(|e| format!("SQLite query error: {}", e))?;

    if rows.is_empty() {
        return Ok(TaleaValue::DbResult { rows: vec![], columns: vec![], count: 0 });
    }

    let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
    let result_rows: Vec<HashMap<String, TaleaValue>> = rows.iter().map(|row| {
        let mut map = HashMap::new();
        for col in row.columns() {
            let name = col.name().to_string();
            let val = sqlite_row_to_talea(row, col.ordinal());
            map.insert(name, val);
        }
        map
    }).collect();

    let count = result_rows.len();
    Ok(TaleaValue::DbResult { rows: result_rows, columns, count })
}

#[cfg(feature = "db")]
fn bind_sqlite<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    val: &TaleaValue,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match val {
        TaleaValue::String(s)  => q.bind(s.clone()),
        TaleaValue::Number(n)  => q.bind(*n),
        TaleaValue::Float(f)   => q.bind(*f),
        TaleaValue::Bool(b)    => q.bind(*b),
        TaleaValue::Null       => q.bind(Option::<String>::None),
        _ => q.bind(format!("{}", val)),
    }
}

#[cfg(feature = "db")]
fn sqlite_row_to_talea(row: &sqlx::sqlite::SqliteRow, idx: usize) -> TaleaValue {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<i64, _>(idx) { return TaleaValue::Number(v); }
    if let Ok(v) = row.try_get::<f64, _>(idx) { return TaleaValue::Float(v); }
    if let Ok(v) = row.try_get::<bool, _>(idx) { return TaleaValue::Bool(v); }
    if let Ok(v) = row.try_get::<String, _>(idx) { return TaleaValue::String(v); }
    TaleaValue::Null
}

#[cfg(feature = "db")]
async fn execute_sqlite(
    pool: &sqlx::SqlitePool,
    sql: &str,
    params: &[TaleaValue],
) -> Result<u64, String> {
    let mut q = sqlx::query(sql);
    for p in params { q = bind_sqlite(q, p); }
    let result = q.execute(pool).await.map_err(|e| format!("SQLite execute error: {}", e))?;
    Ok(result.rows_affected())
}
