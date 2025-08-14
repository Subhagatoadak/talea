
# 🌿 Talea: A Plain-Language Programming Environment for Textual Analysis

**Talea** is a human-first programming language and interactive environment built for natural language processing, textual analysis, and discourse research. It is especially designed for users from the humanities, education, and the social sciences who want to leverage computational power without a steep programming learning curve.

Named after the Latin word *talea*—meaning "a narrative thread" or "a story branch"—Talea's core philosophy is to make complex NLP workflows as intuitive as writing a list of instructions. It achieves this by combining a simple, English-like syntax with the immense power of existing programming ecosystems like Python and R.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ Features

* **📝 Plain-English Syntax:** Write code that reads like instructions, not cryptic symbols. `load "file.txt" as my_text` does exactly what it says.
* **🔌 Pluggable Backends:** Seamlessly use powerful libraries from other languages. The current version integrates with **Python's spaCy** for advanced NLP, with a clear path to add R, Java, and more.
* **📚 Library Imports:** Bring in any Python or R package or script and call its functions directly from Talea.
* **💡 Interactive by Design:** An interactive REPL (Read-Eval-Print Loop) provides immediate feedback on your commands.
* **⚙️ Built-in Logic:** Comes with built-in logic for file I/O, variable manipulation, arithmetic, and data filtering.
* **🌱 Extensible Core:** The modular architecture (Lexer, Parser, Interpreter) makes it easy for contributors to add new commands, parsers, and execution logic.
* **💻 Cross-Platform:** Designed to work on macOS, Linux, and Windows with the right setup.
To learn more, see the [Complete Guide](docs/complete_guide.md) for installation, usage, saving results, and analysis commands.

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

### 4. Run Talea Scripts

Commands can also be stored in a file with the `.talea` extension and executed non-interactively.

```talea
# hello_world.talea
load "article.txt" as article
print article
```

Run the file by passing it to the Talea binary:

```bash
cargo run -- examples/hello_world.talea
```

Lines beginning with `#` are treated as comments and ignored.

### 5. Talea IDE (optional)

A lightweight graphical editor is available in the [`ide/`](ide/README.md) directory. It uses Tauri and the Monaco editor to provide a windowed environment where you can open, edit, run, and save `.talea` scripts. Refer to its [README](ide/README.md) for installation and usage details.

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

### Importing Python and R Libraries

> **Note:** R support requires building Talea with the `r` feature. Run `cargo run --features r` (or `cargo build --features r`) to enable it.

You can load external packages or scripts and call their functions directly:

```talea
> use python
> import python "math" as m
> call m sqrt 16 as py_result
> print py_result
4

> use r
> import r "stats" as s
> call s sqrt 25 as r_result
> print r_result
5
```

### Advanced NLP Use Cases (Python & R)

Talea can orchestrate sophisticated natural language processing pipelines by calling out to the Python and R ecosystems. Below are a few illustrative scenarios:

#### Python: Sentiment Analysis with TextBlob

```talea
> use python
> import python "textblob" as tb
> call tb TextBlob "Talea makes NLP easy" as blob
> call blob sentiment as sentiment
> print sentiment
```

#### Built-in Bag-of-Words

Generate a bag-of-words representation without relying on Python:

```talea
> load "article.txt" as my_article
> tokenize my_article as tokens
> doc2bow tokens as bow
> print bow
```

#### R: Token Frequency with quanteda

```talea
> use r
> import r "quanteda" as q
> call q tokens my_article as r_tokens
> call q dfm r_tokens as counts
> print counts
```

These examples combine Talea's English-like syntax with powerful packages to perform advanced NLP without leaving the REPL.

### Installing Missing Libraries

If a required package isn't present, Talea can install it for you:

```talea
> install python "textblob"
> install r "quanteda"
```

After installation, you can `import` the module and use it immediately. Talea sets the CRAN mirror to `https://cloud.r-project.org` for R packages, so you won't be prompted to choose one.

-----

### Talking to OpenAI in plain English

```rust
use talea::openai_pipeline::OpenAiPipeline;
let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo").unwrap();
let reply = pipeline.ask("Write a short haiku about the sea.").unwrap();
println!("{}", reply);
```

Inside the Talea REPL you can even set environment variables with plain English:

```
> set environment variable OPENAI_API_KEY to "sk-your-key"
> openai "Write a haiku about rust" as poem
> print poem
> openai "Write a haiku about rust" with "gpt-4" to "haiku.txt"
```

The last command asks the `gpt-4` model for a poem and saves the reply into
`haiku.txt` in one step.

-----

### One-line OpenAI Pipeline

If you have an OpenAI API key in the `OPENAI_API_KEY` environment variable,
Talea can generate text and save it straight to a file with a single call:

```rust
use talea::openai_pipeline::OpenAiPipeline;

OpenAiPipeline::quick(
    "Explain the significance of the Renaissance.",
    "renaissance.txt",
).unwrap();
```

The model's reply will be written to `renaissance.txt`.

### Working with Data

Curious about numbers? The friendly [Statistics Guide](docs/statistics.md) walks you through loading datasets, computing descriptive stats, running tests, fitting regressions, and turning results into prose with OpenAI.

-----

## 📚 Command Reference

Talea understands a growing set of plain-English commands. The table below summarizes the currently supported ones:

| Command | Purpose | Example |
|--------|---------|---------|
| `load` | Read a file into a variable | `load "file.txt" as my_text` |
| `tokenize` | Split text into tokens | `tokenize my_text as tokens` |
| `doc2bow` | Convert tokens to bag-of-words | `doc2bow tokens as bow` |
| `split` | Split text by unit (word, sentence, line, character, paragraph) | `split lines in my_text as lines` |
| `join` | Join list items into text | `join words in tokens with " " as sentence` |
| `replace` | Replace text patterns | `replace "cat" with "dog" in my_text as new_text` |
| `clean` | Remove punctuation/whitespace | `clean my_text as cleaned` |
| `normalize` | Normalize text (e.g., lowercase) | `normalize my_text as clean_text` |
| `lowercase/uppercase` | Change text case | `lowercase my_text as lower_text` |
| `stem` | Stem words | `stem my_text as stems` |
| `lemmatize` | Lemmatize text via spaCy | `lemmatize my_text as lemmas` |
| `tag` | NLP tagging with spaCy (NER, POS) | `tag my_text with ner as entities` |
| `summarize` | Summarize text | `summarize my_text as summary` |
| `paraphrase` | Rephrase text using OpenAI | `paraphrase my_text as paraphrased` |
| `translate` | Translate text to another language | `translate my_text to "spanish" as spanish_text` |
| `sentiment` | Analyze sentiment of text | `sentiment my_text as mood` |
| `wordcloud` | Generate word frequency list | `wordcloud my_text as cloud` |
| `association` | Analyze word associations | `association tokens as pairs` |
| `similarity` | Measure semantic similarity between texts | `similarity text1 to text2 as score` |
| `semanticsearch` | Search documents by semantic similarity | `semanticsearch "rust" in docs as results` |
| `search` | Keyword or key phrase search | `search "rust" in my_text as hits` |
| `phrase` | Extract phrases from text | `phrase my_text as phrases` |
| `paragraph` | Work with paragraphs | `paragraph my_text as paras` |
| `concordance` | Find sentences containing a word | `concordance "rust" in my_text as contexts` |
| `count` | Count units in a list or text | `count words in my_text as word_count` |
| `filter` | Filter lists by condition | `filter tokens containing "rust" as rust_tokens` |
| `remove` | Remove items from a list | `remove stopwords from tokens as clean_tokens` |
| `slice` | Slice list by index | `slice tokens from 0 to 5 as first_tokens` |
| `sort` | Sort list items | `sort tokens ascending as sorted_tokens` |
| `group` | Group list items by key | `group tokens by length as groups` |
| `first/last` | Retrieve first or last item | `first token in tokens as start` |
| `sample` | Random sample from list | `sample 10 from tokens as sample_tokens` |
| `define` | Create variables | `define threshold as 10` |
| `set environment variable` | Set OS environment variables | `set environment variable API_KEY to "123"` |
| `today` | Get the current date | `today as current_date` |
| `format date` | Format a date string | `format date current_date with "%Y-%m-%d" as formatted` |
| `parse date` | Parse text into a date | `parse date "2024-01-01" as d` |
| `date diff` | Difference between dates | `date diff start_date and end_date as days` |
| `use` | Choose backend language | `use python` |
| `import` | Import a Python or R module | `import python "math" as m` |
| `install` | Install a Python module or R package | `install python "textblob"` |
| `call` | Invoke a function from an imported module | `call m sqrt 16 as result` |
| `for each` | Loop over lists in a single line | `for each item in items do print item end` |
| `openai` | Query OpenAI models | `openai "Hello" as reply` |
| `add/subtract/multiply/divide` | Arithmetic operations | `add 5 to count as new_count` |
| `save` | Write content to a file | `save results to "out.txt"` |
| `print` | Display a value | `print summary` |

Custom commands can be added in Rust by extending the parser and interpreter, enabling domain-specific workflows.

## 🏗️ Building a macOS Release

Talea ships with a helper script that bundles the compiled binary into a
macOS `.app` and DMG disk image.

```bash
# Run on macOS with Node.js installed
scripts/build_dmg.sh
```

The script installs the IDE's npm dependencies if necessary, builds the
Tauri application, and on macOS opens the bundled Talea IDE so the
interpreter is immediately available. The resulting `dist/Talea-<version>.dmg`
contains the core application. If the `examples/` directory is present it
is archived as `Talea-examples.tar.gz` for an optional download.

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

