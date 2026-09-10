# Command `and`

Hide already visible lines that do not match the pattern/conditions given.

### Python example: show only classes and special `__` methods.

`rxedit pysample.py d:: 'and::__|class '`

#### Output

```
class Calculator:
    def __init__(self, name: str):
```

### Using `wk=...` where key

`rxedit pysample.py d:: d::_::sk=underscores and::::wk=underscores`

This is a bit contrived, but the second `d::` is setting `key=underscores` to all 
declarations that have `_` in their name.

`and` will then remove any lines that do NOT have `key=underscores` set.


#### Output

```
def do_init(func):
    def __init__(self, name: str):
    def show_history(self) -> None:
```

### keep classes as well, but only within first 20 lines



`rxedit pysample.py d:: d::_::sk=keep d::::wt=c.sk=keep and::::wk=keep.wl=-20`

- `d::` - show all declarations
- `d::_::sk=keep` - set any declarations with `_` in their names as key=keep
- `d::::wt=c.sk=keep` - use `wt=c` i..e where type = class to set any class to key=keep
- `and::::wk=keep.wl=-20` only show visible lines where key=keep and - `wl=-1` - within lines 1 to 10.


#### Output:

```
def do_init(func):
class Calculator:
    def __init__(self, name: str):
```


## and flags

|Name                  | Format               | Example        | Description|
|----------------------| ---------------------| ---------------| -----------|
|after                 | A(\d+)               | ::A16          | Show NUM lines after each match.|
|before                | B(\d+)               | ::B5           | Show NUM lines before each match.|
|context               | C(\d+)               | ::C5           | Show NUM lines before and after each match.|
|insensitive           | i                    | ::i            | case-insentive search|
|fixed_string          | F                    | ::F            | treat search as regular string, not a regex|
|use_grep_shortcodes   | g                    | ::g            | expands shortcodes (`..`→`.*`, `~~s.`→` +`)|
|no_use_grep_shortcodes| G                    | ::G            | does not expand grep shortcodes|
|set key               | sk=([a-z]+)(?:\.\|$) |  ::sk=important|  set key to a value that can be used with wk=<key>. in later commands|
|where_key             | wk=([a-z]+)(?:\.\|$) |  ::wk=mykey.   |  puts an extra criteria to other commands|
|where_linenum         | wl=([0-9,-]+)(?:\.\|$|  ::wl=-5,22,99.|  limit line number range|
### SEE ALSO 

  [`shortcodes`](../shortcodes.md), [chaining](../chaining.md), [`noop`](./noop.md)

