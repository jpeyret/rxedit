# Command `explain`

Prints how rxedit processed your command arguments at the very end, to stderr.  

This is useful because rxedit will silently ignore most mistyped commands or flags rather than stopping execution.

For example, let's say you want to show all Python `__init__` constructors *and* their owner, which will be a class.

`rxedit pysample.py d::__init__::O -n`

giving

````
    def __init__(self, name: str):
````

what happened?  you were expecting `::o` to show the owner.

`rxedit pysample.py d::__init__::O explain`

in addition to the stdout output you had before you now get extra stuff on stderr

```
-----------------------------------------
[explain] / processing for these arguments...
Global configuration:
  space between blocks=false, show line numbers=false, grep shortcode substitution:true
  file=pysample.py

d::__init__::O /?ignored: `O`  => [declare] regex: `__init__`
    use_grep_shortcodes
```

The important part is  `?ignored: 'O'`.  It turns out you didn't use lower case `o` so `O`, which means 
nothing for a `declaration` command, was silently ignored. Correcting that, you get the expected output

` rxedit pysample.py d::__init__::o`

```
class Calculator:
    def __init__(self, name: str):
```

Or you want to show all `print` calls, with 2 lines surrounding them:

`rxedit pysample.py more::print:C2 `

but it gives you no output.  Adding `explain` gives extra information

````
-----------------------------------------
[explain] / processing for these arguments...
Global configuration:
  space between blocks=false, show line numbers=false, grep shortcode substitution:true
  file=pysample.py

more::print:C2 => [more] regex: `print:C2`
    use_grep_shortcodes
````

That regex is very fishy and that's because you only specified one `:`.  Using `more::print::C2` fixes the problem.


### SEE ALSO --help for

  `noop`
