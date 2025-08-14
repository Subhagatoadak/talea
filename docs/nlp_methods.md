# NLP Methods in Talea

The table below outlines common NLP methods and their corresponding Talea commands.
This serves as an exhaustive reference for the language. Commands marked as
*Planned* are placeholders for future contributions.

| Method | Command | Status |
| ------ | ------- | ------ |
| Tokenization | `tokenize` | Implemented |
| Bag-of-Words | `doc2bow` | Implemented |
| Cleaning / Punctuation Removal | `clean` | Implemented |
| Lowercasing | `lowercase` | Implemented |
| Uppercasing | `uppercase` | Implemented |
| Normalization | `normalize` | Implemented |
| Stemming | `stem` | Implemented |
| Lemmatization | `lemmatize` | Implemented |
| Part-of-Speech Tagging | `tag ... with pos` | Implemented |
| Named Entity Recognition | `tag ... with ner` | Implemented |
| Word Count | `count words` | Implemented |
| Sentence Count | `count sentences` | Implemented |
| Concordance / KWIC | `concordance` | Implemented |
| Phrase Extraction | `phrase` | Implemented |
| Summarization | `summarize` | Implemented |
| Paraphrasing | `paraphrase` | Implemented |
| TF–IDF | `tfidf` | Planned |
| N‑grams | `ngrams` | Planned |
| Collocations | `collocate` | Planned |
| Sentiment Analysis | `sentiment` | Implemented |
| Dependency Parsing | `parse` | Planned |
| Topic Modeling (LDA) | `topicmodel` | Planned |
| Word Embeddings | `embed` | Planned |
| Similarity / Distance | `similarity` | Planned |
| Translation | `translate` | Implemented |
| Text Generation | `openai` | Implemented |

## Command Usage and Examples

### tokenize
Split text into individual tokens.

**Usage:** `tokenize <text> as <tokens>`

**Example:**
```talea
tokenize article as tokens
```

### doc2bow
Convert tokens into a bag-of-words representation.

**Usage:** `doc2bow <tokens> as <bow>`

**Example:**
```talea
doc2bow tokens as bow
```

### clean
Remove punctuation and extra whitespace.

**Usage:** `clean <text> as <clean_text>`

**Example:**
```talea
clean article as clean_article
```

### lowercase
Convert text or list items to lowercase.

**Usage:** `lowercase <text> as <lower>`

**Example:**
```talea
lowercase article as lower
```

### uppercase
Convert text or list items to uppercase.

**Usage:** `uppercase <text> as <upper>`

**Example:**
```talea
uppercase article as upper
```

### normalize
Normalize text (for example, Unicode normalization).

**Usage:** `normalize <text> as <normalized>`

**Example:**
```talea
normalize article as normalized
```

### stem
Apply a stemming algorithm to words.

**Usage:** `stem <text_or_tokens> as <stems>`

**Example:**
```talea
stem article as stems
```

### lemmatize
Reduce words to their lemmas.

**Usage:** `lemmatize <text_or_tokens> as <lemmas>`

**Example:**
```talea
lemmatize article as lemmas
```

### tag ... with pos
Apply part-of-speech tags.

**Usage:** `tag <text_or_tokens> with pos as <pos_tags>`

**Example:**
```talea
tag article with pos as pos_tags
```

### tag ... with ner
Identify named entities.

**Usage:** `tag <text_or_tokens> with ner as <entities>`

**Example:**
```talea
tag article with ner as entities
```

### count words
Count words in text.

**Usage:** `count words in <text> as <count>`

**Example:**
```talea
count words in article as word_count
```

### count sentences
Count sentences in text.

**Usage:** `count sentences in <text> as <count>`

**Example:**
```talea
count sentences in article as sentence_count
```

### concordance
Find keyword-in-context occurrences.

**Usage:** `concordance "<term>" in <text> as <hits>`

**Example:**
```talea
concordance "rust" in article as hits
```

### phrase
Extract frequent phrases.

**Usage:** `phrase <text_or_tokens> as <phrases>`

**Example:**
```talea
phrase article as phrases
```

### summarize
Create a short summary of text.

**Usage:** `summarize <text> as <summary>`

**Example:**
```talea
summarize article as summary
```

### paraphrase
Reword text.

**Usage:** `paraphrase <text> as <paraphrase>`

**Example:**
```talea
paraphrase article as rephrased
```

### tfidf *(Planned)*
Compute term frequency–inverse document frequency scores.

**Usage:** `tfidf <documents> as <scores>`

**Example:**
```talea
tfidf corpus as scores
```

### ngrams *(Planned)*
Generate n-grams from tokens.

**Usage:** `ngrams <tokens> with n <size> as <grams>`

**Example:**
```talea
ngrams tokens with n 2 as bigrams
```

### collocate *(Planned)*
Find collocations in text.

**Usage:** `collocate <tokens> as <collocations>`

**Example:**
```talea
collocate tokens as collocations
```

### sentiment
Analyze sentiment of text.

**Usage:** `sentiment <text> as <sentiment>`

**Example:**
```talea
sentiment article as mood
```

### parse *(Planned)*
Perform dependency parsing.

**Usage:** `parse <text> as <parse_tree>`

**Example:**
```talea
parse article as tree
```

### topicmodel *(Planned)*
Fit a topic model such as LDA.

**Usage:** `topicmodel <documents> as <model>`

**Example:**
```talea
topicmodel corpus as model
```

### embed *(Planned)*
Generate word embeddings.

**Usage:** `embed <tokens_or_text> as <embeddings>`

**Example:**
```talea
embed tokens as vectors
```

### similarity *(Planned)*
Measure similarity between texts.

**Usage:** `similarity <text1> to <text2> as <score>`

**Example:**
```talea
similarity article to summary as score
```

### translate
Translate text to another language.

**Usage:** `translate <text> to "<language>" as <translation>`

**Example:**
```talea
translate article to "spanish" as es_article
```

### openai
Generate or manipulate text using an OpenAI model.

**Usage:** `openai "<prompt>" as <result>`

**Example:**
```talea
openai "Write a haiku" as poem
```

## Workflow Example

Start the Talea REPL from the project root:

```bash
cargo run
```

Once the prompt appears, the following commands demonstrate a complete
NLP workflow:

```talea
load "article.txt" as article
tokenize article as tokens
tag article with ner as entities
summarize article as summary
save summary to "summary.txt"
```

This snippet loads a text file, splits it into tokens, identifies named
entities, summarizes the content, and writes the result to disk.

Contributions implementing any of the *Planned* commands are welcome.
