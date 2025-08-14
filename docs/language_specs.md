# Talea Language Specification

Talea programs are written in plain English. Each non‑empty line is a statement executed from top to bottom. Lines beginning with `#` are comments and are ignored by the interpreter.

```talea
# Comments start with '#'
load "article.txt" as article
```

## Variables and Expressions

* **Variables** – Results are stored in simple names.
* **Literals** – Strings (`"text"`), numbers (`42`), and lists (`[1,2,3]`).
* **Indexing** – Use brackets to index into lists (`tokens[0]`).
* **Environment variables** – `set environment variable NAME to "value"`.

```talea
define threshold as 10
set environment variable OPENAI_API_KEY to "sk-..."
```

## File I/O

| Command | Example |
| ------- | ------- |
| Load a file | `load "article.txt" as article` |
| Save a value | `save article to "out.txt"` |
| Print a value | `print article` |

## Text Operations

| Command | Example |
| ------- | ------- |
| Tokenize text | `tokenize article as tokens` |
| Split by unit | `split sentences in article as sentences` |
| Join list items | `join words in tokens with " " as sentence` |
| Replace patterns | `replace "cat" with "dog" in article as pets` |
| Clean punctuation and whitespace | `clean article as clean_article` |
| Normalize text (general cleanup) | `normalize article as normalized` |
| Lowercase text | `lowercase article as lower` |
| Uppercase text | `uppercase article as upper` |
| Stem words | `stem article as stems` |
| Lemmatize words | `lemmatize article as lemmas` |

## Analysis Commands

| Command | Example |
| ------- | ------- |
| Count units | `count words in article as word_count` |
| Concordance / KWIC | `concordance "rust" in article as hits` |
| Tag with POS or NER | `tag article with ner as entities` |
| Summarize text | `summarize article as summary` |
| Translate text | `translate article to "spanish" as es_article` |
| Sentiment analysis | `sentiment article as mood` |
| Filter lists | `filter tokens containing "rust" as rust_tokens` |

## Control Flow and Arithmetic

* **Loops** – `for each item in list do ... end`
* **Conditionals** – `if condition then ... else ... end`
* **Definition** – `define name as value`
* **Arithmetic** – `add`, `subtract`, `multiply`, and `divide`

```talea
if word_count is greater than 1000 then print "long article" else print "short article" end

for each t in tokens do
    print t
end

add 5 to word_count as updated
```

## External Integration

Talea can call out to other languages and services.

| Command | Example |
| ------- | ------- |
| Enable backend | `use python` |
| Install module | `install python "textblob"` |
| Import module | `import python "textblob" as tb` |
| Call function | `call tb TextBlob "hi" as blob` |
| OpenAI prompt | `openai "Write a haiku" as poem` |

## Utility Commands

| Command | Purpose | Example |
| ------- | ------- | ------- |
| Help | Show command table | `help` |
| Exit | Leave the interpreter | `exit` |

This specification lists every statement currently understood by the interpreter. Refer to the user guide for a tutorial style walk‑through of these features.
