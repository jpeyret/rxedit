# Commands `prependfile` and `appendfile`

These two commands behave much like a `sed i` and `sed a`, respectively, but instead of inserting some user-provided text, they "include" lines from the provided file.  i.e. when the condition matches the file's contents will be inserted before or after the match line.  
Unlike `sed` however, this command only affects visible lines.



-  notice the`-I [indent_keep]` flag.
- like the `macro` command, `rxedit` will error out if it can't find the file.

#### Example:  show definitions matching "init", then include file's contents before them

`rxedit pysample.py d::/init prependfile::init:: header_for_init_functions.py`

```
class Calculator:
# line1
# line2
    def __init__(self, name: str):

```

#### Example:  Match the existing indentation

`rxedit pysample.py d::/init prependfile::init::I header_for_init_functions.py`

```
class Calculator:
    # line1
    # line2
    def __init__(self, name: str):

```
