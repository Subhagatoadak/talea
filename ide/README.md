# Talea IDE

The Talea IDE is a lightweight desktop editor built with [Tauri](https://tauri.app/) and the [Monaco editor](https://microsoft.github.io/monaco-editor/). It lets you write, run, and save `.talea` scripts with a simple graphical interface.

## Prerequisites

- [Node.js](https://nodejs.org/) and `npm`
- [Rust](https://www.rust-lang.org/) (required by Tauri)

## Installation

```bash
cd ide
npm install
```

This installs the JavaScript dependencies and the Tauri CLI.

## Running the IDE

```bash
npm run tauri dev
```

A window will open with the Talea editor. The toolbar provides:

- **New** – start a fresh buffer
- **Open** – load an existing `.talea` file
- **Save** – write the current editor contents to disk
- **Run Selection** – execute the highlighted lines
- **Run File** – run the entire editor contents
- **Interpreter** – choose a Talea executable to run scripts

## Building a Release

To create a standalone desktop bundle:

```bash
npm run tauri build
```

The generated binary is placed in `src-tauri/target/release` and platform-specific bundles appear under `src-tauri/target/release/bundle`.

## Usage Tips

- Syntax highlighting is provided for common Talea keywords.
- Output from running code appears in the lower console area and scrolls automatically.
- Use standard Talea commands (`load`, `tokenize`, `print`, etc.) inside the editor.
