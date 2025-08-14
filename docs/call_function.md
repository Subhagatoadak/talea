# `call` Function Reference

The `call` command runs functions from imported libraries or methods on objects.
Use it to interact with Python or other backends while writing Talea scripts.


## Two Ways to `call`

There are two patterns depending on what you want to run.

### 1. Call a function from a library

```
call <library> <function> [arguments] as <result>
```

- **`<library>`** – alias from an earlier `import` command.
- **`<function>`** – the function provided by that library.
- **`[arguments]`** – optional values passed to the function.
- **`<result>`** – variable that stores what the function returns.

Example:

## Basic Structure

```
call <source> <function-or-method> [arguments] as <result>
```

- **`<source>`** – library alias or variable name holding an object.
- **`<function-or-method>`** – the function to run or the method on the object.
- **`[arguments]`** – optional values passed to the function/method.
- **`<result>`** – name for the value returned from the call.

## Examples

### Calling a function from a library


```
use python
import python "pandas" as pd
call pd read_csv "examples/sample_data.csv" as data
```


### 2. Call a method on an object

```
call <object> <method> [arguments] as <result>
```

- **`<object>`** – a variable that already holds a Python object.
- **`<method>`** – the action to run on that object.
- **`[arguments]`** – optional values passed to the method.
- **`<result>`** – variable that stores the method’s return value.

Example:

```
# data was created in the previous example

### Calling a method on an object

```

call data describe as summary
print summary
```


If you get `Library 'data' not found. Did you import it?`, Talea could not find the
`data` object. Make sure the earlier call that created `data` succeeded and that
the name is spelled the same.


## Common Errors and Fixes

| Error message | Cause | How to fix |
|---------------|-------|------------|

| `Library 'X' not found. Did you import it?` | Talea cannot find the library or object name | For library calls, import the module. For object methods, make sure the variable exists. |
| `Function 'f' not found in library 'X'` | Wrong function name | Confirm the function or method exists and is spelled correctly. |
| `Missing file or invalid path` | `read_csv` or similar could not find the file | Verify the file path and put it in quotes. |
| `Type mismatch or argument error` | Arguments do not match the function signature | Review the required arguments and provide them in order. |

| `Library 'X' not found. Did you import it?` | `<source>` is unknown | Import the library or check the variable name. |
| `Function 'f' not found in library 'X'` | Wrong function name | Confirm the function or method exists and is spelled correctly. |
| `Missing file or invalid path` | `read_csv` or similar could not find the file | Verify the file path and enclose it in quotes. |
| `Type mismatch or argument error` | Arguments do not match function signature | Review the required arguments and provide them in the correct order. |


## Advanced Calls

- **Multiple arguments:**
  ```
  call math pow 2 3 as eight
  ```
- **Keyword arguments:**
  ```
  call pd DataFrame my_dict index=["a", "b"] as df
  ```
- **Chaining:** first call a function, then call a method on the result.
  ```
  call pd read_csv "examples/sample_data.csv" as data
  call data head 10 as first_rows
  ```

## Troubleshooting Steps

1. Ensure `use python` (or another backend) appears before importing libraries.
2. Verify every imported library has an alias: `import python "pandas" as pd`.
3. When calling object methods, double‑check that the object name matches the one you defined earlier.
4. If an error persists, try printing intermediate results to confirm they hold the expected value.

The `call` command offers a straightforward way to tap into powerful libraries while keeping Talea syntax readable and simple.
