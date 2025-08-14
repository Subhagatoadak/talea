# Talea Statistics Guide

This guide shows how to load datasets, compute descriptive statistics, run common statistical tests, fit regression models, and ask OpenAI for narrative insights. Each section includes short Talea examples; full scripts live in the [`examples/`](../examples) directory.

## Loading a Dataset
The repository includes a small CSV file at [`examples/sample_data.csv`](../examples/sample_data.csv).

### Python
```talea
use python
import python "pandas" as pd
call pd read_csv "examples/sample_data.csv" as data
print data
```

### R
```talea
use r
import r "readr" as readr
call readr read_csv "examples/sample_data.csv" as data
print data
```

## Descriptive Statistics
Generate quick summary statistics with pandas or base R.

### Python
```talea
use python
import python "pandas" as pd
call pd read_csv "examples/sample_data.csv" as data
callmethod data describe as summary
print summary
```

### R
```talea
use r
import r "readr" as readr
import r "base" as base
call readr read_csv "examples/sample_data.csv" as data
call base summary data as summary
print summary
```

## Hypothesis Tests
Run a two-sample t-test comparing groups A and B.

### Python
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

### R
```talea
use r
import r "readr" as readr
import r "stats" as stats
call readr read_csv "examples/sample_data.csv" as data
call stats t_test formula="y ~ group" data=data as result
print result
```

## Regression Analysis
Fit a simple linear regression with statsmodels or base R.

### Python
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

### R
```talea
use r
import r "readr" as readr
import r "stats" as stats
call readr read_csv "examples/sample_data.csv" as data
call stats lm formula="y ~ x" data=data as model
callmethod model summary as summary
print summary
```

## OpenAI Insights
Turn numeric results into prose using OpenAI.

```talea
load "examples/sample_data.csv" as raw_data
openai "Explain the pattern in this dataset:\n" + raw_data as insight
print insight
```

### Example Scripts
Reusable Python scripts demonstrating each section are available:

- [`examples/dataset_loading.tea`](../examples/dataset_loading.tea)
- [`examples/descriptive_stats.tea`](../examples/descriptive_stats.tea)
- [`examples/statistical_tests.tea`](../examples/statistical_tests.tea)
- [`examples/regression.tea`](../examples/regression.tea)
- [`examples/openai_insights.tea`](../examples/openai_insights.tea)
