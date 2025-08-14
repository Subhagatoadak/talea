# Talea Documentation

## Introduction

Talea is a plain‑language programming environment designed for natural language processing (NLP) and text analysis. Instead of relying on traditional syntax filled with symbols and boilerplate, Talea programs read like structured English sentences. This approach lets researchers, educators, and other non‑technical users perform complex computational tasks without learning a conventional programming language.

## Why Talea Was Built

Most programming languages require extensive training before a user can accomplish even simple tasks. Talea was created to remove that barrier. It was built to:

* **Democratize NLP** – give anyone the ability to run state‑of‑the‑art NLP pipelines.
* **Bridge the gap** between humanistic inquiry and computational tools.
* **Provide transparency** by keeping instructions readable and explicit.
* **Leverage existing ecosystems** – Talea can call Python, R, and other languages when necessary.

## Intended Audience

Talea is aimed at:

* Scholars in the humanities and social sciences.
* Teachers who want to introduce computational thinking.
* Journalists, writers, and analysts exploring large corpora.
* Developers who need a quick way to prototype text‑processing pipelines.

## Why Talea Is Better Than Python or R for Non‑Technicals

Python and R are powerful but intimidating. Talea focuses on accessibility:

* **Plain English Syntax** – commands such as `load "file.txt" as article` describe themselves.
* **Immediate Feedback** – the interactive REPL evaluates each instruction as soon as it is entered.
* **Reduced Setup** – users call existing Python/R libraries without writing bridge code.
* **Consistency** – no need to remember indentation rules or symbol‑heavy syntax.
* **Progressive Complexity** – start with simple commands and add advanced logic only when needed.

## Command Reference

The following commands are currently supported by the Talea interpreter. Each command is presented with a short description and an example. This table is also used by the `help` command in the interpreter to keep documentation and runtime output in sync.

| Command | Purpose | Example |
| ------- | ------- | ------- |
| `load` | Read a file into a variable | `load "article.txt" as article` |
| `save` | Write content to a file | `save article to "out.txt"` |
| `print` | Display the contents of a variable | `print article` |
| `tokenize` | Split text into individual tokens | `tokenize article as tokens` |
| `doc2bow` | Convert tokens to bag-of-words | `doc2bow tokens as bow` |
| `split` | Split text by a unit such as words, sentences, lines, or characters | `split sentences in article as sentences` |
| `join` | Join list items into text | `join words in tokens with " " as sentence` |
| `replace` | Replace text patterns | `replace "cat" with "dog" in article as pets` |
| `clean` | Remove punctuation and extra whitespace | `clean article as clean_article` |
| `normalize` | Normalize text (e.g., lowercase, Unicode) | `normalize article as normalized` |
| `lowercase` | Convert text or list items to lowercase | `lowercase article as lower` |
| `uppercase` | Convert text or list items to uppercase | `uppercase article as upper` |
| `stem` | Stem words in text or lists | `stem article as stems` |
| `lemmatize` | Lemmatize words | `lemmatize article as lemmas` |
| `count` | Count units in a list or text | `count words in article as word_count` |
| `concordance` | Find keyword-in-context occurrences | `concordance "rust" in article as hits` |
| `tag` | Apply NLP tagging (POS or NER) | `tag article with ner as entities` |
| `summarize` | Summarize text | `summarize article as summary` |
| `translate` | Translate text to another language | `translate article to "spanish" as es_article` |
| `sentiment` | Analyze sentiment of text | `sentiment article as mood` |
| `filter` | Filter list items by a condition | `filter tokens containing "rust" as rust_tokens` |
| `define` | Create a variable | `define threshold as 10` |
| `set environment variable` | Set an OS environment variable | `set environment variable API_KEY to "123"` |
| `add` | Add two values | `add 5 to count as new_count` |
| `subtract` | Subtract one value from another | `subtract 1 from count as less` |
| `multiply` | Multiply two values | `multiply count by 2 as double` |
| `divide` | Divide one value by another | `divide count by 2 as half` |
| `for each` | Iterate over items in a list | `for each t in tokens do print t end` |
| `if` | Conditional execution | `if count is greater than 10 then print "big" else print "small" end` |
| `use` | Choose the backend language | `use python` |
| `install` | Install a Python module or R package | `install python "textblob"` |
| `import` | Import a module from the chosen backend | `import python "textblob" as tb` |
| `call` | Call a function from an imported module | `call tb TextBlob "Talea rocks" as blob` |
| `openai` | Query an OpenAI model | `openai "Write a haiku" as poem` |
| `help` | Show command reference | `help` |
| `exit` | Leave the interpreter | `exit` |

### Additional Notes

* Commands can be chained by saving intermediate results in variables.
* Arithmetic commands (`add`, `subtract`, `multiply`, `divide`) operate on numbers but also work with text when appropriate (e.g., concatenation via `add`).
* Loops (`for each`) may contain any nested command, including calls to external libraries or OpenAI.

## Installing Python or R Libraries

If a needed module isn't available, Talea can install it directly.
Use the `install` command to fetch a Python module or R package, for example:

```talea
install python "textblob"
install r "quanteda"
```

When an import fails, the interpreter still provides a helpful hint showing which
`install` command to run. For R packages, Talea automatically uses the `https://cloud.r-project.org` CRAN mirror to avoid
interactive prompts.

## Common NLP Techniques in Talea

Talea wraps common NLP techniques in plain English instructions. Below are representative examples.

### Tokenization
```talea
load "article.txt" as article
split sentences in article as sentences
for each s in sentences do
    tokenize s as tokens
    print tokens
end
```

### List Indexing
```talea
load "article.txt" as article
split sentences in article as sentences
print sentences[0]
```
This prints the first sentence. Lists can also be created using brackets, for example `define numbers as [1,2,3]`.

### Normalization and Lemmatization
```talea
load "article.txt" as article
normalize article as clean
lemmatize clean as lemmas
print lemmas
```

### Named Entity Recognition (NER)
```talea
load "article.txt" as article
tag article with ner as entities
print entities
```

### Part‑of‑Speech Tagging
```talea
load "article.txt" as article
tag article with pos as pos_tags
print pos_tags
```

### Summarization
```talea
load "article.txt" as article
summarize article as summary
print summary
```

Talea can summarize plain text directly. To summarize a list of numbers,
enable the R backend first with `use r`.

### Topic Modeling (via Python's `gensim`)
Use the built-in `doc2bow` command to create a bag-of-words representation
without requiring Python dependencies.
```talea
load "article.txt" as article
tokenize article as tokens
doc2bow tokens as bow
print bow
```

### Sentiment Analysis (via `textblob`)
```talea
use python
import python "textblob" as tb
call tb TextBlob "Talea makes NLP easy" as blob
call blob sentiment as sentiment
print sentiment
```

These examples show how Talea combines English‑like commands with powerful libraries to perform real NLP work.

## Using OpenAI from Talea

Talea provides first‑class support for OpenAI's APIs.

### From the REPL
```talea
set environment variable OPENAI_API_KEY to "sk-your-key"
openai "Write a haiku about rust" as poem
print poem
openai "Explain Talea" with "gpt-4" to "explanation.txt"
```
The first command sets the API key, the second stores the model's reply in a variable, and the third writes the response directly to a file while specifying a model.

### From Rust Code
```rust
use talea::openai_pipeline::OpenAiPipeline;

let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo").unwrap();
let reply = pipeline.ask("Write a short haiku about the sea.").unwrap();
println!("{}", reply);
```
The `from_env` constructor reads the API key from `OPENAI_API_KEY` and prepares a pipeline for repeated queries.

### One-line Pipeline
```rust
use talea::openai_pipeline::OpenAiPipeline;

OpenAiPipeline::quick(
    "Explain the significance of the Renaissance.",
    "renaissance.txt",
).unwrap();
```
`quick` sends a prompt and saves the reply to the specified file without needing to manage the pipeline manually.

## Loops Combined with OpenAI

Because `openai` is just another command, it can appear inside loops for batch generation.

```talea
load "ideas.txt" as ideas
split lines in ideas as prompts
for each p in prompts do
    openai p as result
    save result to "out/" + p + ".txt"
end
```
This script reads a list of prompts from `ideas.txt`, queries OpenAI for each prompt, and saves every response to its own file.

Another example shows chaining analysis and generation:

```talea
load "article.txt" as article
split sentences in article as sentences
for each s in sentences do
    summarize s as brief
    openai "Rewrite: " + brief as refined
    print refined
end
```
Here each sentence is summarized and then rewritten by an OpenAI model.

## Conclusion

Talea turns natural language into executable code. By embracing readable commands, seamless library integration, and direct access to services like OpenAI, it lowers the barrier to entry for textual analysis. Whether you are a scholar, teacher, or developer, Talea lets you focus on ideas instead of syntax.

