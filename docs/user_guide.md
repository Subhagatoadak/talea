# Talea User Guide

This guide demonstrates the full feature set of Talea. Each code block
uses plain English instructions that can be entered at the `Talea REPL`
prompt.

## 1. Start the REPL
Run Talea from the project root:

```bash
cargo run
```

## 2. Working with Text
Load, inspect, and save files.

```talea
load "article.txt" as article
print article
save article to "copy.txt"
```

## 3. Transform Text
Common text manipulation commands.

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

## 4. Analyze Text
Extract information from text or lists.

```talea
count words in article as word_count
concordance "rust" in article as hits
tag article with ner as entities
summarize article as summary
filter tokens containing "rust" as rust_tokens
```

## 5. Variables and Arithmetic
Create variables, set environment variables, and do math.

```talea
define threshold as 10
set environment variable OPENAI_API_KEY to "sk-..."
add 5 to threshold as higher
subtract 2 from higher as lower
multiply lower by 3 as scaled
divide scaled by 2 as final_value
```

## 6. Control Flow
Loops and conditionals allow more complex logic.

```talea
for each s in sentences do
    print s
end

if word_count > 100 then
    print "long"
else
    print "short"
end
```

## 7. External Libraries and OpenAI
Talea can use Python or R libraries and query OpenAI models.

```talea
use python
install python "textblob"
import python "textblob" as tb
call tb TextBlob "Talea is great" as blob
openai "Write a haiku" as poem
```

## 8. Save Results
Write any value to disk for later use.

```talea
save summary to "summary.txt"
```

By chaining these commands you can build rich NLP workflows entirely in
plain English.
