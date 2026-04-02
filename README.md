# Talea

Talea is a plain-language programming environment for text analysis, lightweight data work, and research workflows. It combines an English-like language, a Rust runtime, optional Python/R/database integrations, a bytecode compiler/VM, and a small Tauri-based desktop IDE.

You can use Talea interactively in a REPL, run `.tea` or `.talea` scripts, translate prose into executable Talea, or compile scripts to `.taleac` bytecode.

## What Is In The Repo

- `src/`: core lexer, parser, AST, interpreter, compiler, VM, OpenAI helper, memory, and database modules
- `examples/`: sample Talea scripts plus a Rust OpenAI example
- `tests/`: parser and interpreter coverage
- `docs/`: guides for language usage, NLP methods, statistics, and command reference
- `ide/`: Tauri + Monaco desktop editor for working with Talea files

## Current Capabilities

- Plain-English commands for loading files, cleaning text, tokenizing, counting, searching, replacing, splitting, joining, saving, and printing
- Control flow and functions: `if`, `for each`, `while`, `try/catch`, `break`, `continue`, `return`, and user-defined functions
- Parallel execution with Rayon plus named channels for producer/consumer style workflows
- Memory controls such as explicit `free`, memory limits, and scoped allocation blocks
- Python interop for importing modules and calling functions or methods
- Optional R support via `--features r`
- Optional PostgreSQL, MySQL, and SQLite support via `--features db`
- Natural-language to Talea translation with `--english` and `--to-talea`
- Bytecode compilation with `--compile` and execution with `--run`
- OpenAI access from Talea scripts and from the Rust library

## Prerequisites

- A current stable Rust toolchain installed with `rustup`
- A working C toolchain
- Python if you want Python-backed commands or library imports
- Node.js and `npm` only if you want to run the desktop IDE

Platform notes:

- macOS: install Xcode Command Line Tools before building Rust projects
- Linux: install a standard native build toolchain such as `build-essential` or `clang`
- Windows: use the MSVC Rust toolchain and see `windows_user_guide.md` for the current Windows-specific setup

Python notes:

- On macOS and Linux, the root crate tries to download an embedded Python build on first compile unless `TALEA_USE_SYSTEM_PYTHON=1` is set
- If you prefer your own Python installation, set `TALEA_USE_SYSTEM_PYTHON=1` and use your normal PyO3/Python environment setup
- Python-backed examples and tests may require additional packages such as `pandas`, `statsmodels`, or `matplotlib`

## Quick Start

```bash
git clone https://github.com/Subhagatoadak/talea.git
cd talea
cargo run
```

That starts the REPL:

```text
> 
```

On the first build, Talea may spend extra time downloading its embedded Python runtime.

## Running Scripts

Talea accepts both `.tea` and `.talea` source files.

```bash
cargo run -- examples/hello_world.tea
cargo run -- examples/descriptive_stats.tea
```

Minimal example:

```talea
load "article.txt" as article
tokenize article as tokens
count words in article as word_count
print word_count
```

## CLI Modes

Run a Talea script or let Talea auto-detect English prose:

```bash
cargo run -- path/to/script.tea
```

Translate prose to Talea and execute it:

```bash
cargo run -- --english notes.txt
```

Translate prose to Talea without running it:

```bash
cargo run -- --to-talea notes.txt
```

Compile a Talea script to bytecode:

```bash
cargo run -- --compile examples/hello_world.tea
```

Run compiled bytecode:

```bash
cargo run -- --run examples/hello_world.taleac
```

## Optional Features

Enable R support:

```bash
cargo run --features r
```

Enable database support:

```bash
cargo run --features db -- examples/dataset_loading.tea
```

Use OpenAI from Talea by setting `OPENAI_API_KEY`:

```bash
export OPENAI_API_KEY=your_key_here
cargo run
```

Example Talea commands:

```talea
set environment variable OPENAI_API_KEY to "sk-..."
openai "Write a short summary of this text" as summary
print summary
```

## IDE

The desktop editor lives in [`ide/`](ide/) and is documented in [`ide/README.md`](ide/README.md).

```bash
cd ide
npm install
npm run tauri dev
```

The IDE runs Talea through an external interpreter binary. If `talea` is not on your `PATH`, use the IDE's interpreter picker to point it at the built executable.

To build a release bundle:

```bash
./scripts/install_dependencies.sh
```

Artifacts are written under `ide/src-tauri/target/release/bundle`.

On macOS, `./scripts/build_dmg.sh` also copies generated bundles into `dist/`.

## Documentation

- [`docs/complete_guide.md`](docs/complete_guide.md): end-to-end getting started guide
- [`docs/all_in_one_guide.md`](docs/all_in_one_guide.md): consolidated command and workflow reference
- [`docs/user_guide.md`](docs/user_guide.md): short usage walkthrough
- [`docs/statistics.md`](docs/statistics.md): statistical analysis commands and examples
- [`docs/nlp_methods.md`](docs/nlp_methods.md): NLP-related methods
- [`windows_user_guide.md`](windows_user_guide.md): Windows-specific setup

## Development

Run the test suite with:

```bash
cargo test
```

If Rust fails during linking on macOS with an `xcode-select` or missing developer tools error, install Xcode Command Line Tools first.

## License

Talea is distributed under the MIT license. See `LICENSE`.
