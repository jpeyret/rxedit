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

Note how using `A2` on the initial [`declarations`](./declarations.md) command can allow the user
to determine how much leading whitespace to use.

```
class Calculator:
    def __init__(self, name: str):
        "init function"
        # a comment
        self.name = name
```


## prepend/append flags

|Name                  | Format             | Example    | Description|
|----------------------| -------------------| -----------| -----------|
|insensitive           | i                  | ::i        | case-insentive search|
|indent_keep           | I                  | ::I        | keep matching line's indentation|
|fixed_string          | F                  | ::F        | treat search as regular string, not a regex|
|use_grep_shortcodes   | g                  | ::g        | expands shortcodes (`..`→`.*`, `~~s.`→` +`)|
|no_use_grep_shortcodes| G                  | ::G        | does not expand grep shortcodes|
|where_key             | wk=([a-z]+)(?:\.\|$|  ::wk=mykey|  puts an extra criteria to other commands|
|user_confirm          | u                  | ::u        | user confirmation|
### Notes about commands that modify the text

- Nothing gets saved until you use the `-O` modify in place flag or the `-o <some path>` output flag.
- Those commands only apply to matching visible lines, unlike say a `sed` insert or delete.
- `append`,`prepend`,`delete` will throw off the line numbers used in `where line` flags
- When modifying a source code file, the result may not be syntaxically correct, so be careful.
  - rxedit is designed on the expectation that *modification* commands will be used on version-controlled files.
  - but you could use the `-o myfile.bak` flag to save and diff your changes first.
- Use [`explain`](./explain.md) to make sure all previous commands to show/hide lines went as expected.
- Modification commands support the `::u` user confirmation flag, which will prompt the user for confirmation.  
  - Use of that flag will, by design, trigger an immediate error in batch mode.
- rxedit will "follow" symlinks so it will affect the actual file if you modify it.

### SEE ALSO 

  [`shortcodes`](../shortcodes.md), [chaining](../chaining.md), [`noop`](./noop.md), [flags](../flags.md)

