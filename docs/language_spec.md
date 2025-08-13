# Talea Grammar Overview

Talea programs are plain-English instructions.  Each line is a statement, and comments start with `#`.

```talea
# A comment
load "article.txt" as article
```

## Variables

Variables are simple names where results are stored.

```talea
load "article.txt" as article
normalize article as clean_article
```

## Core Statements

### Loading text
```talea
load "article.txt" as article
```

### Transforming text
```talea
split sentences in article as sentences
count words in article as word_count
```

### Using external tools
```talea
use python
import python "textblob" as tb
call tb TextBlob "Talea is great" as blob
```

### Output
```talea
print word_count
save article to "out.txt"
```

## Control Flow

Loops use `for each`/`end` and may contain any statements.

```talea
for each s in sentences do
    print s
end
```

This overview covers the most common pieces of Talea's grammar.  Detailed command references and advanced features live in other documents.
