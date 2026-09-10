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
