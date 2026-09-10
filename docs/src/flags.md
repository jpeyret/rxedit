# Using flags

This covers a few special flags available on many commands separately



#### `sk=` set_key and `wk=` where_key

set key `sk=<value>.` and where key  `wk=<value>.`

Set key allows to mark affected lines with a "key" of some value, which can be referred to by 
later commands as an additional criterion.  An example of how it works:

` rxedit pysample.py more::init::sk=mykey`

All lines matching "init" are shown and set to `key=mykey`

```
def do_init(func):
    def __init__(self, name: str):
    @do_init
```

next, [`declarations`](./commands/declarations.md) will show all class and method lines.

`rxedit pysample.py more::init::sk=mykey declarations`

giving

```
def do_init(func):
class Calculator:
    def __init__(self, name: str):
    def add(self, a: int, b: int) -> int:
    def multiply(self, a: int,
                b: int) -> int:
    @do_init
    def show_history(self) -> None:
```

let's now use `::wk=mykey` to hide only lines with that key set.

` rxedit pysample.py more::init::sk=mykey declarations less::o::wk=mykey`

that [`less`](./commands/less.md) will match any line with "o" in it, but only if had been marked with `key=mykey`.  So "show_history" is kept, but not "do_init".

```
class Calculator:
    def __init__(self, name: str):
    def add(self, a: int, b: int) -> int:
    def multiply(self, a: int,
                b: int) -> int:
    def show_history(self) -> None:
```

#### Setting the value for the flag.

For these types of arbitrary value flags, value is whatever comes after the `=`, until a `.` or the end of the flag.

` more::init::sk=mykey.A1` and ` more::init::sk=mykey` will both set the key to "mykey".  

Note that the lines made visible by `A1` are made visible, but are not set to "mykey".

#### `wl=` where linenum flag

This add the extra criterion that the lines must be in the specified line range(s).

`rxedit -n pysample.py d::::wl=-10,30-`

```
000005 def do_init(func):
000009 class Calculator:
000031     def show_history(self) -> None:
```

#### `u` user confirmation flag

For potentially dangerous commands or when you want to handle lines one by one, the `u` user confirmation flag asks you to confirm.

`rxedit pysample.py d::init delete::::u`

```
> delete def do_init(func): Yes
? delete     def __init__(self, name: str): (Y/n)
...
```

Using this flag triggers an abort if rxedit sees it is executed in a batch, rather than interactively.

