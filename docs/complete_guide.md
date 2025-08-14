# Talea Complete Guide

Talea is a plain-language programming environment for textual analysis. This guide collects everything you need to get started, run the interpreter, save results, and perform all available analyses in one place.

## Introduction

Talea lets you write instructions in simple English to manipulate text, call external libraries, and query OpenAI models. Commands look like sentences and run interactively in a REPL or from `.talea` script files.

## Installation and Setup

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Install Python and spaCy**
   ```bash
   conda create --name talea-nlp python=3.11
   conda activate talea-nlp
   pip install -U spacy
   python -m spacy download en_core_web_md
   ```
3. **Clone the repository and configure the linker**
   ```bash
   git clone https://github.com/your-username/talea.git
   cd talea
   # create .cargo/config.toml pointing to your conda env lib path
   ```

## Running Talea

Activate your conda environment and start the REPL from the project root:

```bash
conda activate talea-nlp
cargo run
```

To run a script saved in `examples/`:

```bash
cargo run -- examples/hello_world.talea
```

## Working with Files

Load, inspect, and save text with plain-English commands.

```talea
load "article.txt" as article
print article
save article to "copy.txt"
```

## Text Transformation

Common operations for cleaning and reshaping text.

```talea
tokenize article as tokens
split sentences in article as sentences
join words in tokens with " " as rebuilt
replace "cat" with "dog" in article as pets
clean article as clean_article
normalize article as normalized
lowercase article as lower
uppercase article as upper
stem article as stems
lemmatize article as lemmas
```

## Analysis Commands

Extract insights and generate new text.

```talea
count words in article as word_count
concordance "rust" in article as hits
tag article with ner as entities
summarize article as summary
translate article to "spanish" as es_article
sentiment article as mood
openai "Write a haiku about rust" as poem
save summary to "summary.txt"
```

## Variables, Arithmetic, and Control Flow

Use variables, math, loops, and conditionals to build richer workflows.

```talea
define threshold as 10
add 5 to threshold as higher
if word_count is greater than threshold then print "long" else print "short" end
for each s in sentences do
    print s
end
```

## External Libraries and Statistics

Talea can call Python packages for data work.

```talea
use python
import python "pandas" as pd
call pd read_csv "examples/sample_data.csv" as data
callmethod data describe as summary
print summary
```

You can also run statistical tests or regressions by importing additional libraries like `scipy` or `statsmodels`.

## Command Reference

| Command | Purpose |
| ------- | ------- |
| `load` | Read a file into a variable |
| `save` | Write content to a file |
| `print` | Display the contents of a variable |
| `tokenize` | Split text into individual tokens |
| `split` | Divide text by words, sentences, lines, or characters |
| `join` | Join list items into text |
| `replace` | Replace text patterns |
| `clean` | Remove punctuation and extra whitespace |
| `normalize` | Normalize text |
| `lowercase` / `uppercase` | Change text case |
| `stem` | Apply a stemming algorithm |
| `lemmatize` | Reduce words to their lemmas |
| `count` | Count units in a list or text |
| `concordance` | Keyword-in-context search |
| `tag` | Apply POS or NER tagging |
| `summarize` | Summarize text |
| `translate` | Translate to another language |
| `sentiment` | Analyze sentiment |
| `wordcloud` | Generate a word-frequency list |
| `association` | Analyze word associations |
| `similarity` | Measure semantic similarity |
| `semanticsearch` | Search documents semantically |
| `search` | Keyword search |
| `filter` | Filter list items by condition |
| `define` | Create variables |
| `set environment variable` | Set an OS environment variable |
| `add` / `subtract` / `multiply` / `divide` | Arithmetic |
| `for each` / `if` | Loops and conditionals |
| `use` | Choose backend language |
| `install` / `import` / `call` | Work with external libraries |
| `openai` | Query OpenAI models |
| `exit` | Leave the interpreter |

With these commands you can build end‑to‑end analyses entirely in plain English. Run them interactively or save scripts for reproducible workflows.

