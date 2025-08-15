# Talea All-in-One Guide

This document gathers the most commonly used Talea commands, natural language processing (NLP) methodologies, statistical workflows, and common errors in one place. Each section includes the command structure, usage, examples, and potential pitfalls.

## 1. Core Command Reference

### `load`
* **Purpose:** Read file contents into a variable.
* **Usage:** `load "<file_path>" as <variable>`
* **Example:** `load "article.txt" as article`
* **Possible Errors:**
  * *FileNotFound* – path is wrong or file missing.
  * *PermissionDenied* – insufficient permissions to read the file.

### `save`
* **Purpose:** Write the value of a variable to disk.
* **Usage:** `save <variable> to "<file_path>"`
* **Example:** `save article to "copy.txt"`
* **Possible Errors:**
  * *FileNotFound* – destination directory does not exist.
  * *PermissionDenied* – no permission to write to destination.

### `print`
* **Purpose:** Display the value stored in a variable.
* **Usage:** `print <variable>`
* **Example:** `print article`
* **Possible Errors:**
  * *UndefinedVariable* – variable name has not been defined.

### `tokenize`
* **Purpose:** Split text into individual tokens.
* **Usage:** `tokenize <text> as <tokens>`
* **Example:** `tokenize article as tokens`
* **Possible Errors:**
  * *TypeError* – source value is not text.

### `split`
* **Purpose:** Divide text by words, sentences, lines, characters, or paragraphs.
* **Usage:** `split <unit> in <text> as <list>` where `<unit>` is `words`, `sentences`, `lines`, `characters`, or `paragraphs`.
* **Example:** `split sentences in article as sentences`
* **Possible Errors:**
  * *InvalidUnit* – unsupported split unit.

### `join`
* **Purpose:** Join items from a list into text.
* **Usage:** `join <unit> in <list> with "<separator>" as <text>`
* **Example:** `join words in tokens with " " as rebuilt`
* **Possible Errors:**
  * *TypeError* – source is not a list of strings.

### `replace`
* **Purpose:** Replace text patterns.
* **Usage:** `replace "<pattern>" with "<replacement>" in <text> as <new_text>`
* **Example:** `replace "cat" with "dog" in article as pets`
* **Possible Errors:**
  * *RegexError* – invalid regular expression pattern.

### `clean`
* **Purpose:** Remove punctuation and extra whitespace.
* **Usage:** `clean <text> as <clean_text>`
* **Example:** `clean article as clean_article`
* **Possible Errors:**
  * *TypeError* – value provided is not text.

### `normalize`
* **Purpose:** Normalize text (e.g., Unicode normalization).
* **Usage:** `normalize <text> as <normalized>`
* **Example:** `normalize article as normalized`
* **Possible Errors:**
  * *TypeError* – non-text input.

### `lowercase` / `uppercase`
* **Purpose:** Change the case of text.
* **Usage:** `lowercase <text> as <lower>` or `uppercase <text> as <upper>`
* **Example:** `lowercase article as lower`
* **Possible Errors:**
  * *TypeError* – non-text input.

### `stem`
* **Purpose:** Apply a stemming algorithm.
* **Usage:** `stem <text_or_tokens> as <stems>`
* **Example:** `stem article as stems`
* **Possible Errors:**
  * *MissingModel* – stemming backend not available.

### `lemmatize`
* **Purpose:** Reduce words to their lemmas.
* **Usage:** `lemmatize <text_or_tokens> as <lemmas>`
* **Example:** `lemmatize article as lemmas`
* **Possible Errors:**
  * *MissingModel* – spaCy model not installed.

### `tag`
* **Purpose:** Apply POS or NER tagging.
* **Usage:** `tag <text_or_tokens> with pos|ner as <tags>`
* **Example:** `tag article with ner as entities`
* **Possible Errors:**
  * *MissingModel* – spaCy model not installed.
  * *InvalidTagger* – unsupported tagger type.

### `count`
* **Purpose:** Count words, sentences, or list items.
* **Usage:** `count <unit> in <text_or_list> as <count>` where `<unit>` is `words`, `sentences`, etc.
* **Example:** `count words in article as word_count`
* **Possible Errors:**
  * *InvalidUnit* – unsupported count unit.

### `concordance`
* **Purpose:** Keyword-in-context search.
* **Usage:** `concordance "<term>" in <text> as <results>`
* **Example:** `concordance "rust" in article as hits`
* **Possible Errors:**
  * *TermNotFound* – term absent from text.

### `summarize`
* **Purpose:** Summarize text via OpenAI.
* **Usage:** `summarize <text> as <summary>`
* **Example:** `summarize article as summary`
* **Possible Errors:**
  * *APIError* – OpenAI API key missing or request failed.

### `translate`
* **Purpose:** Translate text to another language.
* **Usage:** `translate <text> to "<language>" as <translation>`
* **Example:** `translate article to "spanish" as es_article`
* **Possible Errors:**
  * *APIError* – OpenAI API key missing or invalid.
  * *UnsupportedLanguage* – translation target not recognized.

### `sentiment`
* **Purpose:** Analyze sentiment of text.
* **Usage:** `sentiment <text> as <sentiment>`
* **Example:** `sentiment article as mood`
* **Possible Errors:**
  * *MissingModel* – sentiment analyzer not installed.

### `define`
* **Purpose:** Create a variable.
* **Usage:** `define <name> as <value>`
* **Example:** `define threshold as 10`
* **Possible Errors:**
  * *SyntaxError* – missing keyword `as` or value.

### `for each`
* **Purpose:** Iterate over list items.
* **Usage:** `for each <item> in <list> do <commands> end`
* **Example:** `for each s in sentences do print s end`
* **Possible Errors:**
  * *TypeError* – variable is not a list.

### `if`
* **Purpose:** Conditional execution.
* **Usage:** `if <condition> then <commands> else <commands> end`
* **Example:** `if word_count is greater than 10 then print "long" else print "short" end`
* **Possible Errors:**
  * *SyntaxError* – missing `end` or malformed condition.

### `use`, `import`, `install`, `call`, `callmethod`
* **Purpose:** Interoperate with Python or R libraries.
* **Usage:**
  * `use python` or `use r`
  * `install python "<package>"`
  * `import python "<module>" as <alias>`
  * `call <alias> <function> <args> as <var>`
  * `callmethod <object> <method> <args> as <var>`
* **Example:**
  ```talea
  use python
  import python "pandas" as pd
  call pd read_csv "examples/sample_data.csv" as data
  callmethod data describe as summary
  print summary
  ```
* **Possible Errors:**
  * *ImportError* – module not installed or wrong alias.
  * *CallError* – function or method name misspelled.

### `openai`
* **Purpose:** Send a prompt to an OpenAI model.
* **Usage:** `openai "<prompt>" as <variable>`
* **Example:** `openai "Write a haiku about rust" as poem`
* **Possible Errors:**
  * *APIError* – missing key, invalid parameters, or network issue.

## 2. NLP Methodologies

Talea offers high-level commands for many NLP tasks. Below is a summary with usage, examples, and potential errors.

| Method | Command | Usage | Example | Possible Errors |
| ------ | ------- | ----- | ------- | ---------------- |
| Tokenization | `tokenize` | `tokenize <text> as <tokens>` | `tokenize article as tokens` | Non-text input |
| Bag-of-Words | `doc2bow` | `doc2bow <tokens> as <bow>` | `doc2bow tokens as bow` | Tokens not a list |
| Cleaning | `clean` | `clean <text> as <clean>` | `clean article as clean_article` | Non-text input |
| Lowercasing | `lowercase` | `lowercase <text> as <lower>` | `lowercase article as lower` | Non-text input |
| Uppercasing | `uppercase` | `uppercase <text> as <upper>` | `uppercase article as upper` | Non-text input |
| Normalization | `normalize` | `normalize <text> as <norm>` | `normalize article as normalized` | Non-text input |
| Stemming | `stem` | `stem <text> as <stems>` | `stem article as stems` | Missing stemmer |
| Lemmatization | `lemmatize` | `lemmatize <text> as <lemmas>` | `lemmatize article as lemmas` | spaCy model missing |
| POS Tagging | `tag ... with pos` | `tag <text> with pos as <tags>` | `tag article with pos as pos_tags` | spaCy model missing |
| NER | `tag ... with ner` | `tag <text> with ner as <entities>` | `tag article with ner as entities` | spaCy model missing |
| Word Count | `count words` | `count words in <text> as <count>` | `count words in article as word_count` | Non-text input |
| Sentence Count | `count sentences` | `count sentences in <text> as <count>` | `count sentences in article as sentence_count` | Non-text input |
| Concordance | `concordance` | `concordance "term" in <text> as <results>` | `concordance "rust" in article as hits` | Term not found |
| Summarization | `summarize` | `summarize <text> as <summary>` | `summarize article as summary` | API errors |
| Sentiment Analysis | `sentiment` | `sentiment <text> as <sentiment>` | `sentiment article as mood` | Model not installed |
| Translation | `translate` | `translate <text> to "lang" as <translation>` | `translate article to "spanish" as es_article` | API or language errors |

*Planned features* like `tfidf`, `ngrams`, and `topicmodel` are not yet implemented but can be added by contributors.

## 3. Statistical Methods

Talea integrates with Python and R for data analysis.

### Loading Data
```talea
use python
import python "pandas" as pd
call pd read_csv "examples/sample_data.csv" as data
```
*Possible Errors:* file not found, pandas not installed.

### Descriptive Statistics
```talea
callmethod data describe as summary
print summary
```
*Possible Errors:* `data` not a DataFrame.

### Hypothesis Testing (two-sample t-test)
```talea
use python
import python "pandas" as pd
import python "scipy.stats" as stats
call pd read_csv "examples/sample_data.csv" as data
callmethod data query "group == 'A'" as group_a
callmethod data query "group == 'B'" as group_b
callmethod group_a __getitem__ "y" as y_a
callmethod group_b __getitem__ "y" as y_b
call stats ttest_ind y_a y_b as result
print result
```
*Possible Errors:* groups empty, SciPy not installed.

### Regression Analysis
```talea
use python
import python "warnings" as warnings
import python "pandas" as pd
import python "statsmodels.formula.api" as smf
call warnings filterwarnings "ignore"
call pd read_csv "examples/sample_data.csv" as data
call smf ols "y ~ x" data as model
callmethod model fit as fitted
callmethod fitted summary as summary
print summary
```
*Possible Errors:* statsmodels not installed, malformed formula.

### Built-in Talea Regression
```talea
load "examples/sample_data.csv" as data
regress y using x in data as model
```
*Possible Errors:* file missing, non-numeric columns.

### OpenAI Insights
```talea
load "examples/sample_data.csv" as raw_data
openai "Explain the pattern in this dataset:\n" + raw_data as insight
print insight
```
*Possible Errors:* API key missing, large dataset exceeding prompt limits.

## 4. Common Errors and Troubleshooting

| Error | Cause | Fix |
| ----- | ----- | --- |
| `UndefinedVariable` | Variable used before assignment | Ensure variable is defined with `load`, `define`, or command output. |
| `FileNotFound` | File path incorrect | Check spelling and relative vs absolute paths. |
| `MissingModel` | spaCy or other NLP model not installed | Run `pip install spacy` and download required models. |
| `ImportError` | Python/R package missing | Use `install python "package"` or `install r "package"`. |
| `APIError` | OpenAI key missing or network issue | Set `OPENAI_API_KEY` and check connectivity. |
| `TypeError` | Command received wrong value type | Verify previous commands produced expected data types. |
| `SyntaxError` | Command format invalid | Follow the usage patterns exactly; check for missing keywords. |
| `CallError` | Function or method invocation failed | Confirm function name and arguments. |

## 5. Additional Resources

* `docs/complete_guide.md` – step-by-step setup and REPL instructions.
* `docs/nlp_methods.md` – detailed table of NLP features.
* `docs/statistics.md` – extended statistical walkthroughs.
* `examples/` – runnable Talea scripts showing full workflows.

This document aims to serve as the definitive quick reference for Talea. Contributions that expand examples, cover more commands, or document new methods are welcome!
