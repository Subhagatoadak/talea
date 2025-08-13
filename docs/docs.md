# Command Reference

| Command | Purpose | Example |
| ------- | ------- | ------- |
| `load` | Read a file into a variable | `load "article.txt" as article` |
| `save` | Write content to a file | `save article to "out.txt"` |
| `print` | Display the contents of a variable | `print article` |
| `tokenize` | Split text into individual tokens | `tokenize article as tokens` |
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
