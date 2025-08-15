# Talea All‑in‑One Guide

*A comprehensive, practical, and math‑backed handbook for building text analytics and data workflows with Talea.*

---

## 0) Read This First — How to Use This Guide

* **Who it’s for:** Researchers, data scientists, and students who want powerful NLP + stats workflows with natural, readable commands.
* **What you get:**

  * An expanded **command reference** with patterns and pitfalls
  * **NLP methods** with plain‑English intuition **plus the math**
  * **Statistics & modeling** with end‑to‑end recipes
  * **Interop**: call Python/R seamlessly; simple OpenAI & Gemini usage
  * **Troubleshooting** and reproducibility guardrails
* **Style:** Examples are copy‑pasteable. Look for ✅ **Good** vs ⚠️ **Pitfall** callouts.

---

## 1) Quickstart: From Zero to First Results

```talea
load "examples/article.txt" as article
clean article as article_clean
split sentences in article_clean as sents
for each s in sents do print s end

summarize article_clean as tl; print tl

use python
import python "pandas" as pd
call pd DataFrame {"text": sents} as df
callmethod df head as head5
print head5
```

**What happened?** You loaded text → cleaned → sentence‑split → summarized → moved into a Pandas DataFrame for further stats.

---

## 2) Language Basics: Values, Names, and Types

Talea is **strongly typed at runtime** with friendly coercions where safe.

* **Core value kinds:**

  * `text` (Unicode string)
  * `number` (int/float)
  * `bool` (true/false)
  * `list<text>` / `list<number>` / ...
  * `table` (when coming from Python/R DataFrame)
  * `opaque` (handles to Python/R objects; printed as summaries)
* **Variable definition:** `define name as <value>`
* **Interpolation:** use `+` to concatenate strings: `print "Hello, " + name`
* **Immutability by convention:** Reuse new names for transformed values for clarity (`article → article_clean → sents`).

**Naming tips**

* snake\_case for variables: `word_count`, `clean_article`
* nouns for data, verbs for actions in comments

---

## 3) Core Command Reference (Expanded)

### `load`

* **Purpose:** Read file contents or data into a variable.
* **Usage:** `load "<file_path>" as <variable>`
* **Examples:**

  * Text file → text: `load "article.txt" as article`
  * CSV (raw text): `load "data.csv" as raw_csv` (parse with Python/R if needed)
* **Notes:** Paths are relative to current working directory. Use absolute paths for reliability in batch runs.
* **Possible Errors:** *FileNotFound*, *PermissionDenied*.
* **Pitfall:** Hidden BOM/encoding. If you see strange characters, run `normalize` first.

### `save`

* **Purpose:** Serialize a variable to disk as text (or via Python for tables).
* **Usage:** `save <variable> to "<file_path>"`
* **Examples:** `save article to "copy.txt"`
* **Possible Errors:** *FileNotFound* (directory), *PermissionDenied*.
* **Pattern:** For DataFrames, export via Python: `callmethod df to_csv "out.csv" as _`

### `print`

* **Purpose:** Show a human‑readable representation of a variable.
* **Notes:** For large values, prefer sampling (e.g., `take first 10` using Python interop) to avoid flooding the console.

### `tokenize`

* **Purpose:** Split text into tokens (words by default with Unicode‑aware rules).
* **Usage:** `tokenize <text> as <tokens>`
* **Options (planned):** `tokenize <text> as <tokens> using <lang>`
* **Pitfall:** Numbers & hyphenated words may split unexpectedly; use `normalize` or `replace` beforehand.

### `split`

* **Purpose:** Divide text by `words | sentences | lines | characters | paragraphs`.
* **Usage:** `split <unit> in <text> as <list>`
* **Notes:** Sentence splitting uses rule‑based heuristics; abbreviations like “Dr.” may need post‑fix.

### `join`

* **Purpose:** Merge a list of strings into text with a separator.
* **Usage:** `join words in <list> with " " as <text>`
* **Pitfall:** Mixed types in list → *TypeError*. Ensure a `list<text>` first.

### `replace`

* **Purpose:** Regex/text replacement.
* **Usage:** `replace "<pattern>" with "<replacement>" in <text> as <new_text>`
* **Notes:** Patterns are regular expressions by default. Escape special chars as needed.

### `clean`

* **Purpose:** Remove punctuation, collapse whitespace, standardize quotes/dashes.
* **Usage:** `clean <text> as <clean_text>`
* **Best for:** Bag‑of‑words, frequency counts, concordance prep.

### `normalize`

* **Purpose:** Unicode normalization (NFC), homoglyph resolution (where safe).
* **Usage:** `normalize <text> as <normalized>`

### `lowercase` / `uppercase`

* **Purpose:** Case folding.

### `stem`

* **Purpose:** Apply a stemming algorithm (e.g., Porter or Snowball via Python interop behind the scenes).
* **Possible Errors:** *MissingModel*. Use `install python "nltk"` and `import` a stemmer if needed.

### `lemmatize`

* **Purpose:** Lemmatize tokens using spaCy.
* **Notes:** Requires a spaCy model (e.g., `en_core_web_sm`).
* **Install pattern:**

  ```talea
  use python
  install python "spacy"
  import python "spacy" as spacy
  call spacy load "en_core_web_sm" as nlp   # ensure model is available in environment
  ```

### `tag`

* **Purpose:** POS or NER tagging.
* **Usage:** `tag <text_or_tokens> with pos|ner as <tags>`
* **Pitfall:** Check model language vs text language.

### `count`

* **Purpose:** Count words, sentences, or list items.
* **Usage:** `count <unit> in <text_or_list> as <count>`
* **Tip:** For unique counts, interop with Python sets.

### `concordance`

* **Purpose:** Keyword‑in‑context (KWIC) with window.
* **Usage:** `concordance "term" in <text> as <results>`
* **Post‑processing:** Turn KWIC into a table (see recipes below).

### `summarize`

* **Purpose:** LLM‑based abstractive summary using your OpenAI key.
* **Usage:** `summarize <text> as <summary>`
* **Tips:** Chunk long docs first and `join` summaries.

### `translate`

* **Purpose:** Translate to target language via LLM.
* **Usage:** `translate <text> to "<language>" as <translation>`
* **Note:** For legal/medical contexts, add a style guardrail to the prompt.

### `sentiment`

* **Purpose:** Polarity or valence analysis (rule‑based or model).
* **Tip:** For domain text (e.g., reviews), calibrate with labeled examples via Python.

### `define`

* **Purpose:** Bind a literal or expression to a variable.
* **Usage:** `define threshold as 10`

### `for each`

* **Purpose:** Iterate a list.
* **Usage:** `for each <item> in <list> do <commands> end`
* **Pattern:** Build new lists by appending via Python interop, or collect to a DataFrame.

### `if`

* **Purpose:** Conditional execution.
* **Usage:** `if <condition> then <commands> else <commands> end`
* **Comparators:** `equals`, `not equals`, `greater than`, `less than`, etc.

### Interop: `use`, `install`, `import`, `call`, `callmethod`

* **Purpose:** Seamlessly execute Python/R code and use their libraries.
* **Minimal flow:**

  ```talea
  use python
  install python "pandas"
  import python "pandas" as pd
  call pd read_csv "examples/sample_data.csv" as df
  callmethod df head as h
  print h
  ```
* **Common pitfalls:** Wrong alias, misspelled `callmethod` target, object not a DataFrame.

### `openai`

* **Purpose:** Quick prompt to OpenAI model (uses default model unless configured).
* **Usage:** `openai "Write a haiku about rust" as poem`
* **Configuration:** Ensure `OPENAI_API_KEY` is set in your environment.

---

## 4) Patterns, Recipes, and Anti‑Patterns

**A. Clean → Tokenize → Count**

```talea
load "examples/article.txt" as article
clean article as c
tokenize c as toks
count words in c as n_words
print "Words: " + n_words
```

**B. KWIC Table**

```talea
concordance "sustainability" in c as hits
use python
import python "pandas" as pd
call pd DataFrame hits as kwic
callmethod kwic head as _top
print _top
```

**C. Chunk‑then‑Summarize Long Docs**

```talea
split paragraphs in article as paras
for each p in paras do summarize p as s; print s end
join paragraphs in paras with "\n\n" as joined  # fallback summary if needed
```

**D. Save Cleaned Outputs**

```talea
save c to "outputs/article_clean.txt"
```

**Anti‑Patterns**

* Over‑cleaning (e.g., removing diacritics needed for meaning).
* Treating lists as text in `join`/`replace`.
* Blindly applying English models to multilingual text.

---

## 5) NLP Methodologies — Intuition + Mathematics + Talea Mappings

> **Note:** Some features (e.g., `tfidf`, `ngrams`, `topicmodel`) are **planned** as native commands. Use Python/R interop now, keeping identical semantics so your scripts won’t change later.

### 5.1 Tokenization

* **Goal:** Convert text to meaningful units (tokens).
* **Challenges:** Contractions, hyphenation, emojis, CJK segmentation.
* **Math context:** A tokenization function is a mapping $f: \mathcal{T} \to \mathcal{V}^*$ where $\mathcal{T}$ is text space and $\mathcal{V}$ a vocabulary set.
* **Talea:** `tokenize text as tokens`
* **Interop:** `nltk.word_tokenize`, `spacy` docs.

### 5.2 Bag‑of‑Words (BoW)

* **Definition:** Represent a document as counts of vocabulary terms.
* **Math:** For document $d$ and term $t$, term frequency $\mathrm{tf}(t,d) = \#(t,d)$.
* **Sparse vector:** $\mathbf{x}_d \in \mathbb{R}^{|V|}$ where $[\mathbf{x}_d]_i = \mathrm{tf}(t_i, d)$.
* **Talea now:**

  ```talea
  tokenize article as toks
  doc2bow toks as bow    # if not available, see Python recipe below
  ```
* **Python fallback:** scikit‑learn `CountVectorizer`.

### 5.3 TF–IDF

* **Why:** Down‑weight ubiquitous words.
* **Formulas:**

  * $\text{tfidf}(t,d,D) = \mathrm{tf}(t,d) \cdot \mathrm{idf}(t,D)$
  * $\mathrm{idf}(t,D) = \log\frac{N + 1}{\mathrm{df}(t) + 1} + 1$, where $N$ = # docs, $\mathrm{df}(t)$ = # docs containing $t$.
* **Python fallback:** `TfidfVectorizer(ngram_range=(1,2), min_df=2)`
* **Talea mapping (planned):** `tfidf tokens as X`

### 5.4 n‑grams & Collocations

* **Intuition:** Capture local phrase structure.
* **PMI (association):** $\operatorname{PMI}(w_1,w_2) = \log \frac{p(w_1,w_2)}{p(w_1)p(w_2)}$
* **Recipe (Python):** compute bigrams, keep PMI>0.

### 5.5 Lemmatization vs Stemming

* **Stemming:** heuristic chop (`connect`→`connect`). Fast, noisy.
* **Lemma:** dictionary base form (`better`→`good`). Needs models.
* **Effect:** Lemmas improve alignment across inflections.

### 5.6 POS Tagging

* **Task:** Assign syntactic categories (NOUN, VERB…).
* **Model:** Sequence tagging (e.g., CRF, BiLSTM, Transformer). Conditional probability $p(\mathbf{y}|\mathbf{x})$ over tag sequence.

### 5.7 Named Entity Recognition (NER)

* **Task:** Detect & label spans (PER, ORG, LOC…).
* **Training objective:** Token‑level cross‑entropy over BIO tags.

### 5.8 Sentiment Analysis

* **Patterns:** Rule lexicons (VADER), classical ML, or transformers.
* **Metric:** Polarity score in $[-1,1]$ or discrete {neg, neu, pos}.

### 5.9 Topic Modeling

* **LDA (Latent Dirichlet Allocation):**

  * Generative story: For document $d$, draw topic proportions $\theta_d \sim \operatorname{Dir}(\alpha)$, for each token choose topic $z_{dn} \sim \operatorname{Cat}(\theta_d)$, then word $w_{dn} \sim \operatorname{Cat}(\beta_{z_{dn}})$.
  * **Inference:** Variational Bayes or Gibbs. Output: per‑doc topic mix and per‑topic top words.
* **NMF:** Factorize TF–IDF matrix $X \approx WH$, with $W,H \ge 0$. Optimize $\min_{W,H \ge 0} \|X-WH\|_F^2$.
* **Talea (planned):** `topicmodel using lda on tokens as topics`
* **Python fallback:** `sklearn.decomposition.NMF`, `gensim.models.LdaModel`.

### 5.10 Embeddings (Word/Sentence)

* **Word2Vec (Skip‑gram):** Maximize $\sum_{(w,c)} \log p(c|w)$ with negative sampling.
* **Transformers:** Sentence embeddings via mean pooling of contextual token vectors.
* **Use:** Semantic search, clustering, similarity.

### 5.11 Summarization (Extractive vs Abstractive)

* **Extractive:** Rank sentences by centrality (TextRank: PageRank on sentence similarity graph).
* **Abstractive:** Sequence‑to‑sequence with attention/transformers, often via LLMs (Talea `summarize`).

### 5.12 Translation (LLM‑backed)

* **Add constraints:** “preserve terminology”, “formal tone”, etc.
* **Quality checks:** Round‑trip translate; back‑translation difference.

---

## 6) Statistics & Modeling — Concepts, Math, and Playbooks

### 6.1 Data Loading & EDA

```talea
use python
import python "pandas" as pd
call pd read_csv "examples/sample_data.csv" as df
callmethod df info as _info; print _info
callmethod df describe as _desc; print _desc
```

**Checks:** missingness, type inference, outliers (IQR), duplicates.

### 6.2 Hypothesis Testing

* **Two‑sample t‑test (independent, equal variances):**

  * Test $H_0: \mu_A = \mu_B$.
  * Pooled SE: $s_p^2 = \frac{(n_A-1)s_A^2 + (n_B-1)s_B^2}{n_A+n_B-2}$,
    $t = \frac{\bar{x}_A-\bar{x}_B}{\sqrt{s_p^2(1/n_A+1/n_B)}}$.
* **Welch’s t‑test:** when variances unequal.
* **Effect size:** Cohen’s d = $(\bar{x}_A-\bar{x}_B)/s_p$.
* **Talea + Python:**

  ```talea
  use python
  import python "scipy.stats" as stats
  call stats ttest_ind y_a y_b as res
  print res
  ```

### 6.3 Linear Regression (OLS)

* **Model:** $y = X\beta + \varepsilon,\ \varepsilon \sim \mathcal{N}(0, \sigma^2 I)$
* **Estimator:** $\hat{\beta} = (X^\top X)^{-1} X^\top y$
* **Assumptions:** linearity, independence, homoscedasticity, normality of residuals.
* **Diagnostics:** $R^2$, adjusted $R^2$, residual plots, VIF.
* **Talea:**

  ```talea
  load "examples/sample_data.csv" as data
  regress y using x in data as model   # built‑in wrapper
  ```
* **Python detailed:** `statsmodels.formula.api.ols("y ~ x1 + x2", data=df)`

### 6.4 Logistic Regression

* **Model:** $p(y=1|x) = \sigma(\beta^\top x) = 1/(1+e^{-\beta^\top x})$
* **Loss:** Negative log‑likelihood (cross‑entropy).
* **Use:** Classification (sentiment positive vs negative, etc.).

### 6.5 Regularization

* **Ridge:** $\min_\beta \|y-X\beta\|_2^2 + \lambda\|\beta\|_2^2$
* **Lasso:** $\min_\beta \|y-X\beta\|_2^2 + \lambda\|\beta\|_1$
* **Elastic Net:** mixture of both; useful for correlated predictors.

### 6.6 Cross‑Validation & Model Selection

* **k‑fold CV**; **Stratified** for classification.
* **Metrics:** MSE, MAE, Accuracy, AUROC, F1.

### 6.7 Time Series Essentials

* **Decomposition:** $y_t = T_t + S_t + R_t$ (trend, seasonality, remainder)
* **Double Exponential Smoothing (Holt):**

  * Level: $\ell_t = \alpha y_t + (1-\alpha)(\ell_{t-1} + b_{t-1})$
  * Trend: $b_t = \beta(\ell_t - \ell_{t-1}) + (1-\beta)b_{t-1}$
  * Forecast: $\hat{y}_{t+h} = \ell_t + h b_t$
* **Holt‑Winters (seasonal):** additive or multiplicative season.
* **Intervals:** Use **state space** variance or **bootstrapping** residuals when closed‑forms unavailable.

### 6.8 Bootstrapping

* **Idea:** Resample with replacement, compute statistic across B replicates.
* **CI (percentile):** $[\theta_{(\alpha/2)},\ \theta_{(1-\alpha/2)}]$ from bootstrap empirical distribution.

### 6.9 Bayesian Analysis (PyMC)

* **Posterior:** $p(\theta|y) \propto p(y|\theta)p(\theta)$
* **Workflow:** model → priors → sampling (NUTS) → diagnostics (R‑hat, ESS) → posterior predictive.
* **Interop skeleton:**

  ```talea
  use python
  install python "pymc"
  import python "pymc" as pm
  # build model in Python; keep data frames created in Talea
  ```

### 6.10 MMM (Marketing Mix Modeling) Sketch

* **S‑curve (Hill):** $f(x) = \frac{x^\eta}{\theta^\eta + x^\eta}$
* **Adstock (geometric):** $a_t = x_t + \lambda x_{t-1} + \lambda^2 x_{t-2}+\cdots$
* **Regression:** sales \~ adstocked/spend\_trasnforms + price + promo + controls; estimate ROI and elasticity.
* **Interop:** build with `pymc`/`statsmodels` from Talea tables.

---

## 7) Interoperability: Python & R — Deep Dive

### 7.1 Installing & Importing

```talea
use python
install python "pandas"
import python "pandas" as pd
```

**Note:** Packages install to the environment Talea is configured to use.

### 7.2 Calling Functions and Methods

```talea
call pd read_csv "examples/sample_data.csv" as df
callmethod df query "group == 'A'" as A
callmethod A __getitem__ "y" as yA
```

### 7.3 Moving Data Between Talea and Python

* **Text/List → DataFrame:** `call pd DataFrame {"text": sents} as df`
* **DataFrame → CSV:** `callmethod df to_csv "out.csv" as _`

### 7.4 Creating Your **Own Libraries** (Pluggable Utilities)

* **Goal:** Anyone can ship a small Python/R module; Talea uses it with `import`.
* **Pattern (Python):**

  1. Create `talea_utils/__init__.py` with plain functions.
  2. Install locally: `pip install -e .`
  3. In Talea:

     ```talea
     use python
     import python "talea_utils" as tu
     call tu my_function arg1 arg2 as out
     ```
* **Guideline:** Keep functions **pure** (input → output) and avoid global state for reproducibility.

### 7.5 Simple **Gemini** & **OpenAI** Usage

* **OpenAI (built‑in):** `openai "<prompt>" as out` (requires `OPENAI_API_KEY`).
* **Gemini (via Python):**

  ```talea
  use python
  install python "google-generativeai"
  import python "google.generativeai" as genai
  call genai configure {"api_key": ENV.GEMINI_API_KEY} as _
  call genai GenerativeModel "gemini-1.5-flash" as g
  callmethod g generate_content "Summarize this: " + article as gem_out
  print gem_out
  ```
* **Tip:** Store keys as environment vars (`OPENAI_API_KEY`, `GEMINI_API_KEY`).

---

## 8) End‑to‑End Workflows

### 8.1 Research Paper Digest (Multi‑Doc)

```talea
# Load & normalize
load "papers/paper1.txt" as p1
load "papers/paper2.txt" as p2
normalize p1 as p1n; normalize p2 as p2n

# Clean & split
clean p1n as p1c; clean p2n as p2c
split paragraphs in p1c as p1_paras
split paragraphs in p2c as p2_paras

# Summarize each paragraph then merge
for each q in p1_paras do summarize q as s; print s end
for each r in p2_paras do summarize r as s; print s end

# Build KWIC for a keyword across corpus
join paragraphs in p1_paras with "\n\n" as P1
join paragraphs in p2_paras with "\n\n" as P2
concordance "transformer" in P1 as k1
concordance "transformer" in P2 as k2

# Export tables
use python
import python "pandas" as pd
call pd DataFrame k1 as kwic1
call pd DataFrame k2 as kwic2
callmethod kwic1 to_csv "out/kwic_paper1.csv" as _
callmethod kwic2 to_csv "out/kwic_paper2.csv" as _
```

### 8.2 Sentiment + Drivers (Classification & Interpretation)

```talea
use python
import python "pandas" as pd
import python "sklearn.model_selection" as ms
import python "sklearn.linear_model" as lm
import python "sklearn.metrics" as skm

call pd read_csv "reviews.csv" as df
callmethod df __getitem__ "text" as X
callmethod df __getitem__ "label" as y

# Vectorize (TFIDF)
import python "sklearn.feature_extraction.text" as fet
call fet TfidfVectorizer {"min_df":2,"ngram_range":(1,2)} as vec
callmethod vec fit_transform X as Xv

# Split & train
call ms train_test_split Xv y {"test_size":0.2,"random_state":42} as split
# split returns a tuple; depending on binding, fetch items via Python helper or inline steps

# Logistic regression
call lm LogisticRegression {"max_iter":1000} as clf
callmethod clf fit split[0] split[2] as _
callmethod clf predict split[1] as yhat
call skm classification_report split[3] yhat as rep
print rep
```

### 8.3 Time‑Series Forecast with Holt

```talea
use python
import python "pandas" as pd
import python "statsmodels.tsa.holtwinters" as hw
call pd read_csv "sales.csv" as s
call s set_index "date" as s
callmethod s sort_index as s

call hw ExponentialSmoothing s {"trend":"add"} as mod
callmethod mod fit {"optimized":true} as fit
callmethod fit forecast 12 as fc
print fc
```

---

## 9) Reproducibility, Performance, and Style

### 9.1 Reproducibility

* Pin random seeds in Python (`numpy.random.seed(42)`), log Talea version.
* Save intermediate artifacts (`save`) to make runs restartable.

### 9.2 File Hygiene

* Prefer **absolute** paths for production.
* Avoid spaces in file/dir names; use hyphens/underscores.

### 9.3 Performance Tips

* Stream large files: split into paragraphs first.
* Cache embeddings or vectorizers as Python pickles for reuse.
* Use sparse matrices for BoW/TF‑IDF.

### 9.4 Style

* Small, composable steps; name every intermediate.
* Keep raw, clean, and modeled data **separate**.

---

## 10) Error Handling & Troubleshooting (Extended)

| Error               | Likely Cause              | Diagnosis                               | Fix                                      |
| ------------------- | ------------------------- | --------------------------------------- | ---------------------------------------- |
| `UndefinedVariable` | Misspelled or not defined | `print` variables; check creation order | Define earlier; keep names consistent    |
| `FileNotFound`      | Wrong path / missing dir  | `print CWD` via Python; list files      | Correct path; create directories         |
| `PermissionDenied`  | OS perms                  | Try saving to home; check chmod         | Write to user‑writable path              |
| `MissingModel`      | spaCy/NLTK not installed  | `pip show spacy`; test import           | `install python "spacy"`; download model |
| `ImportError`       | Wrong package name        | Try importing in Python REPL            | Correct package; check alias             |
| `APIError`          | No key / network issue    | Test with minimal prompt                | Set env vars; retry with smaller input   |
| `TypeError`         | Wrong input type          | `print type(var)` via Python            | Convert to list/text as needed           |
| `RegexError`        | Bad pattern               | Test in regex101; escape properly       | Fix expression                           |
| `CallError`         | Method/func not found     | `dir(object)` in Python to inspect      | Correct function or upgrade package      |

**Debug flow:**

1. Reproduce with a minimal script
2. Print types and dimensions
3. Isolate interop calls and test in Python only
4. Add checks (`count`, `if`) before heavy steps

---

## 11) Frequently Asked Questions (FAQ)

**Q1: Can I run multi‑language pipelines?**

> Yes. Mix Talea + Python + R. Keep language blocks clean and pass data via variables.

**Q2: How do I handle very long documents for LLMs?**

> Chunk → summarize each → summarize the summaries. Keep evidence paragraphs next to claims.

**Q3: Is there a way to cache models?**

> Yes, through Python: save vectorizers/models with `joblib` and reload.

**Q4: How do I ensure privacy when using APIs?**

> Remove PII in a preprocessing pass; redact via `replace` before calling `openai`/Gemini.

---

## 12) Command Cheatsheet (One‑Liners)

* **Word frequency (top‑20):**

  ```talea
  clean article as c; tokenize c as t
  use python; import python "collections" as col
  call col Counter t as ctr; callmethod ctr most_common 20 as top
  print top
  ```
* **Unique sentence count:** `split sentences in article as s; define u as s; print count of u` (or dedupe in Python)
* **KWIC export:** see §4B
* **Fast TF‑IDF:** use scikit‑learn (see §8.2)

---

## 13) Roadmap (Planned Native Commands)

* `tfidf`, `ngrams`, `topicmodel`, `embed`, `similarity`, `kwic table`, `csvload` → DataFrame, `plot` (simple hist/line), `assert` for guards.

---

## 14) Appendix — Math at a Glance

* **Cosine similarity:** $\cos(\theta) = \frac{\mathbf{x}\cdot\mathbf{y}}{\|\mathbf{x}\|\,\|\mathbf{y}\|}$
* **Jaccard similarity (sets):** $J(A,B) = \frac{|A\cap B|}{|A\cup B|}$
* **KL divergence:** $D_{\mathrm{KL}}(P\|Q) = \sum_i P_i\log\frac{P_i}{Q_i}$
* **Cross‑entropy:** $H(P,Q) = -\sum_i P_i \log Q_i$
* **MAP estimate:** $\hat{\theta}_{MAP} = \operatorname{argmax}_\theta\ p(y|\theta)p(\theta)$
* **AIC:** $2k - 2\ln(\hat{L})$; **BIC:** $k\ln n - 2\ln(\hat{L})$

---

## 15) Additional Resources

* `docs/complete_guide.md` — setup, REPL, environment config
* `docs/nlp_methods.md` — extended method notes & references
* `docs/statistics.md` — worked examples (OLS, GLM, CV, TS)
* `examples/` — runnable scripts

---

### License & Contributions

This guide is open to contributions: add examples, fix typos, propose new native commands with clear semantics aligned to interop fallbacks. Keep examples minimal, data‑backed, and reproducible.
