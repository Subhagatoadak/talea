# Talea User Guide

This walkthrough mirrors a small research task: finding which places are mentioned in a text.

## 1. Start the REPL
Run Talea from the project root:

```bash
cargo run
```

You will see a `Talea REPL` prompt.

## 2. Load the text
The repository contains `article.txt` with sample prose.  Load it into a variable:

```talea
load "article.txt" as article
```

## 3. Tag named entities
Ask spaCy to recognize names and places:

```talea
tag article with ner as entities
```

## 4. Filter for locations
The NER step returns pairs like `["Berlin", "GPE"]`.  Keep only the geographic entities:

```talea
filter entities where second item is "GPE" as places
```

## 5. Inspect the results
Display the places and how many times each appears:

```talea
print places
count places as place_count
print place_count
```

## 6. Save your findings
If you want a record of the locations:

```talea
save places to "places.txt"
```

This simple workflow shows how Talea scripts map directly to research questions.  By chaining readable commands, you can explore texts, run NLP pipelines, and capture results without leaving plain English.
