
# 🌿 Talea: A Plain-Language Programming Environment for Textual Analysis

**Talea** is a human-first programming language and interactive environment built for natural language processing, textual analysis, and discourse research. It is especially designed for users from the humanities, education, and the social sciences who want to leverage computational power without a steep programming learning curve.

Named after the Latin word *talea*—meaning "a narrative thread" or "a story branch"—Talea's core philosophy is to make complex NLP workflows as intuitive as writing a list of instructions. It achieves this by combining a simple, English-like syntax with the immense power of existing programming ecosystems like Python and R.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ Features

* **📝 Plain-English Syntax:** Write code that reads like instructions, not cryptic symbols. `load "file.txt" as my_text` does exactly what it says.
* **🔌 Pluggable Backends:** Seamlessly use powerful libraries from other languages. The current version integrates with **Python's spaCy** for advanced NLP, with a clear path to add R, Java, and more.
* **💡 Interactive by Design:** An interactive REPL (Read-Eval-Print Loop) provides immediate feedback on your commands.
* **⚙️ Built-in Logic:** Comes with built-in logic for file I/O, variable manipulation, arithmetic, and data filtering.
* **🌱 Extensible Core:** The modular architecture (Lexer, Parser, Interpreter) makes it easy for contributors to add new commands, parsers, and execution logic.
* **💻 Cross-Platform:** Designed to work on macOS, Linux, and Windows with the right setup.

---

## 🚀 Getting Started

Follow these steps to get the Talea development environment running on your machine.

### 1. Prerequisites

You will need the following software installed on your system:

1.  **Rust:** Talea's core is built in Rust. Install it via [rustup.rs](https://rustup.rs/).
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
    ```
2.  **Python:** Talea uses Python for its advanced NLP capabilities. We recommend installing it with Conda for the best experience, but any Python 3.8+ installation will work.
    * **Recommended:** Install [Miniconda](https://docs.conda.io/projects/miniconda/en/latest/) or [Anaconda](https://www.anaconda.com/download).

3.  **Python NLP Libraries:** Once Python is installed, you need `spacy` and its English model.
    ```bash
    # It's best practice to create a dedicated environment
    conda create --name talea-nlp python=3.11
    conda activate talea-nlp

    # Install spaCy and its model
    pip install -U spacy
    python -m spacy download en_core_web_md
    ```

### 2. Clone & Configure the Project

1.  **Clone the Repository:**
    ```bash
    git clone [https://github.com/your-username/talea.git](https://github.com/your-username/talea.git)
    cd talea
    ```

2.  **Configure the Linker:** This is a crucial one-time setup step to tell Rust where to find your Python library.
    * **macOS/Linux:**
        * First, activate your conda environment: `conda activate talea-nlp`
        * Find your Python library path: `echo $CONDA_PREFIX/lib`
        * Create a file named `.cargo/config.toml` in the project root.
        * Add the following content, replacing the example path with the one from the `echo` command:
            ```toml
            # .cargo/config.toml
            [build]
            rustflags = [
                "-L", "/path/to/your/conda/env/lib", # e.g., /opt/anaconda3/envs/talea-nlp/lib
                "-C", "link-arg=-Wl,-rpath,/path/to/your/conda/env/lib" # Add this line for macOS/Linux
            ]
            ```
    * **Windows:**
        * The setup is similar but typically requires linking to the `python3.lib` file. Create `.cargo/config.toml` and add:
            ```toml
            # .cargo/config.toml
            [build]
            rustflags = ["-L", "C:\\path\\to\\your\\conda\\env\\libs"]
            ```

### 3. Build and Run

With the prerequisites and configuration in place, running Talea is simple.

```bash
# Make sure your conda environment is active!
conda activate talea-nlp

# Build and run the Talea REPL
cargo run
````

You should be greeted by the Talea prompt: `Talea REPL v0.2.0 (Python-Powered)`.

-----

## 📖 Example Workflow

Here is a sample session demonstrating some of Talea's capabilities.

```talea
# Create a text file named article.txt with some content first.

> load "article.txt" as my_article
[Interpreter: Successfully read 153 bytes from 'article.txt']

# Use Python/spaCy to perform Named Entity Recognition
> tag my_article with ner as entities
[Interpreter: Calling Python/spaCy for NER tagging...]
[Interpreter: Tagging complete.]

> print entities
[List with 6 items]:
[Tuple([String("Evelyn Reed"), String("PERSON")]), Tuple([String("Berlin"), String("GPE")]), ...]

# Use Python/spaCy to get the base form (lemma) of each word
> lemmatize my_article as lemmas
[Interpreter: Calling Python/spaCy for lemmatization...]
[Interpreter: Lemmatization complete.]

> print lemmas
[List with 31 items]:
[String("Dr."), String("Evelyn"), String("Reed"), String(","), String("a"), ...]

# Create a numeric variable and perform calculations
> define my_count as 150
> multiply my_count by 2 as new_count
> print new_count
300

# Save your results to a file
> save entities to "results.csv"
[Interpreter: Successfully saved content to 'results.csv']
```

-----

## ❤️ Contributing

Contributions are welcome\! Talea is an ambitious project, and help is needed in all areas.

  * **Adding Commands:** The easiest way to contribute is to add a new command. Follow the pattern in `parser.rs` and `interpreter.rs`.
  * **Implementing Logic:** Find a command that is parsed but not yet implemented (like `sort` or `find`) and add the execution logic.
  * **Improving the FFI:** Add support for more Python/R libraries or improve the existing bridge.
  * **Documentation:** Improving this README, adding tutorials, or documenting the language specification in the `/docs` folder is incredibly valuable.
  * **Bug Fixes & Error Handling:** Improve the user-friendliness of runtime and parse errors.

Please feel free to open an issue to discuss your ideas or submit a pull request.

-----

## 📜 License

Talea is distributed under the terms of the MIT license. See [LICENSE](LICENSE.md) for details.

