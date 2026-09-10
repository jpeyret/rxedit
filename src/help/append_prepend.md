# Commands `prepend` and `append`

These tow commands behave much like a `sed i` and `sed a`, respectively.  
i.e. when the condition matches a new line will be inserted before or after the match line.  
Unlike `sed` however, this command only affects visible lines.

Take note of the `-I [indent_keep]` flag.

#### Example:  show definitions matching "init", then add a hello comment before them.

`rxedit pysample.py d::/init 'prepend::init::#hello'`

```
class Calculator:
#hello
    def __init__(self, name: str):
```

#### Example:  Match the existing indentation

` rxedit pysample.py d::/init::A2 'append::init:: "init function"::I'`

Note how using `A2` on the initial `declarations` command can allow the user
to determine how much leading whitespace to use.

```
class Calculator:
    def __init__(self, name: str):
        "init function"
        # a comment
        self.name = name
```
