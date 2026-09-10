rxedit is a folding "batch editor" for source code, inspired by IBM's XEDIT editor, 
optimized for source code discovery and with optional editing capabilities.

Commands progressively show or hide lines, using regular expressions.
Editing operations - loosely modelled on `sed` - affect only visible lines and 
can include user confirmation.
For supported languages, tree-sitter provides added functionality.
Edits are not persisted until requested by the user.

Example:  show all lines containing `TODO` and 2 lines afterwards, using the `A2` flag (similar to a `grep -A 2`)
, hide "low priority" lines and show case-insentitive "urgent":

  `rxedit myfile.txt all::TODO::A2 'less::low priority' more::urgent::i`

#### Visibility commands:
- [`all`](./commands/all.md)          show only matching lines
- [`more`](./commands/more.md)         show more matching lines
- [`less`](./commands/less.md)         hide matching lines
- [`invert`](./commands/invert.md)       invert visibility
- [`lines`](./commands/lines.md)        displays lines within ranges, similar to a posix head or tail

#### Editing commands:
- [`delete`](./commands/delete.md)       delete visible, matching, lines
- [`change`](./commands/change.md)       regex replacement on visible lines.  Supports the full Rust regex syntax.
- [`prepend`](./commands/prepend.md)      add a line before the matching line
- [`append`](./commands/append.md)       add a line after the matching line

#### Language-dependent commands (will become no-ops if the language is not supported)

(currently supports: `.py` and `.rs` extensions)

- [`declarations`](./commands/declarations.md)  look for class, struct or function names that match.
- [`imports`](./commands/imports.md)       look for include, import or use lines that match the name

#### Utility commands:

- [`macro`](./commands/macro.md)        loads commands from a given file, allowing predefinition of search and exclusion patterns
- [`explain`](./commands/explain.md)      stderr what rxedit has understood, and rejected, from user input.


#### Flags (not all commands support all flags, see their individual help)

- `i` case insensitive.
- `F` fixed string, meaning that "string contains" will be used, rather than a regex match.
- `A10`, `B2`, `C1` - similar to using `grep -A 10 -B 2`...
- `o` owner flag - shows the function or class signature to which a matching lines belongs
- `b` body flag - only avaiable on [`declarations`](./commands/declarations.md) commands, shows all lines for a function or class.
- `wt=fs`. For [`declarations`](./commands/declarations.md) only:  filters the types of definitions we are interested in.  On Rust, `fs`
   would match `fn` or `struct`.
- `u` user confirmation, match by match, on [`delete`](./commands/delete.md) or [`change`](./commands/change.md) commands.
- `sk=debug.`  Sets `key=debug` on matching lines.  This can be used to constrain further commands using `wk`.
- `wk=debug.`  In addition to any match conditions, the command will only affect lines with `key=debug` set. 
- `ln=1-10.` In addition to any match conditions the command will only affect the first 10 lines.

Flags are designed to be mixed freely together in any order:  

`more::debug::A2sk=devonly.i` will recognize `A2`, case insensitive and setting `key=devonly` on matching lines.


#### Error handling

Unknown commands, malformed flags, and unsupported language-specific operations are treated as no-ops 
to allow for quicker experimentation.

Use [`explain`](./commands/explain.md) to see how rxedit interpreted your command sequence.  And fine-tune your visible lines
before adding editing commands,


### Command line syntax:

```
Usage: rxedit [OPTIONS] [FILE_PATH] [COMMANDS]...

Arguments:
  [FILE_PATH]    
  [COMMANDS]...  

Options:
  -h, --help [<TOPIC>]                             Print help (optionally for a topic)
  -f, --commands-file <COMMANDS_FILE>              prepend commands from the given file
  -F, --commands-file-after <COMMANDS_FILE_AFTER>  append commands from the given file
  -o, --output-file <OUTPUT_FILE>                  write buffer to the given output file.  Note that everything gets written, not just the visible lines.
  -O, --in-place-output                            modify file directly
  -s, --spacer                                     separate blocks of displayed lines
  -n, --line-number                                show line numbers (those will diverge from the source file change on [`delete`](./commands/delete.md), [`append`](./commands/append.md) or [`prepend`](./commands/prepend.md) use)
  -N, --suppress-line-number                       don't show line numbers (those will diverge from the source file change on [`delete`](./commands/delete.md), [`append`](./commands/append.md) or [`prepend`](./commands/prepend.md) use)
  -v, --verbose                                    
      --debug                                      debug messages
      --dump-json                                  print the parsed hashtree JSON and exit
  -g, --grep-shortcodes                            expands shortcodes (`..`→`.*`,`~~s.`→` +` ...)
  -G, --no-grep-shortcodes                         does not expand grep shortcodes
      --gen-config                                 generate user config file if it does not exist
  -c, --config <FILE>                              load config from the given file
```

Common workflow:
  1. Show lines      (all, more)
  2. Hide lines      (less)
  3. Edit/delete     (change, delete)
  4. Save            (-O or -o)

Commands use:
  COMMAND::PATTERN::FLAGS


### Examples:

  Show all lines containing `TODO` and 2 lines afterwards, using the `A2` flag
  Equivalent to `grep -A 2 TODO myfile.txt`
  `rxedit myfile.txt all::TODO::A2`

  Show all lines with `error`, hide those with `log`, also show those with `print` in upper or lower case.
  [`explain`](./commands/explain.md) prints how rxedit interpreted the commands out to stderr.
    `rxedit myfile.txt -n a::error:: less::log more::print::i explain`

  Show all fn, trait... with a name matching regex `m[ay]` which are not under a parent `tests`
  as in `mod tests {...}`).  The trailing `::b` is the flag that shows those functions' bodies.
  The quotes are necessary to avoid shell interactions.
    `rxedit main.rs 'd::-tests/m[ay]::b'`

  Shows all lines with `dbg!` or `print` and deletes visible lines matching `dbg!`.
  The `-O` flag modifies the file in place, use it after getting the command sequence right.
    `rxedit formatters.rs 'm::dbg!|print' 'delete::dbg!' -O`

  Shows all lines with `dbg!` and uses [`change`](./commands/change.md) via regex group names to comment them out.
  there is no -O yet, so you can fine tune what you are commenting out first.
    `rxedit formatters.rs 'm::dbg!' 'change::([\s]*)([^\s].*$)::$1//$2'`

  Uses a `macro::` file to show user-determined indicators of "chattiness" in Python
    `rxedit pysample.py macro::./showpychatty.rxi`

    showpychatty.rxi contents:

    ````
    #these are typical debugging prints to console
    m::^\s*(print|debug|pp|inspect)\(
    #and these will hide lines that are already commented out.
    less::^\s*#
    ````

  Same as above, after user has hidden lines 32 and 35 that they want to keep after all, delete and overwrite the file
    `rxedit pysample.py macro::./showpychatty.rxi less::::ln=32,35 delete -O`

  Extract all the active `log` calls to a report file.  Note the use of the [`invert`](./commands/invert.md) before [`delete`](./commands/delete.md):  first we
  identify all the lines we want to extract, then we invert the visibility in order to delete all those we don't want.
    `rxedit main.py 'm::log\.[a-z]+\('  'less:: *//' invert delete -o logging.rpt`


### Keep in mind:

- By default, nothing is saved, but `-O` allows modifying the file in place while `-o <path>` allows writing
to a file.

  - Both types of outputs write everything that is in the buffer, visible or not.

  - rxedit makes no assumption about the correctness of code changes.  Deleting or commenting out code may cause errors.
  Preferably, use it on version-controlled files.

- Tree-sitter is used to recognize functions, classes and imports, provided the file extension matches
  a known language (currently, .rs and .py).
  Commands like `declarations::` will result in no-ops when there is no matching grammar.

- As much as possible, user error in entering commands and command flags will be ignored and will result in
  do-nothing operations and ignored flag directives.

  -  Use the [`explain`](./commands/explain.md) command to get feedback on what it understood or not.

  -  Risk: If you display lots of lines, then want to use [`less`](./commands/less.md) to hide some before editing what's left
  this could be a problem.  For example, if you mistyped `less:keep-this!` (notice the mistyped `::` field separator)
  , immediately followed by a [`delete`](./commands/delete.md).


─────────────────────────────
### REMINDER

Visibility:
  all            replace visible set
  more           add matches
  less           remove matches
  invert         invert visible set

Editing:
  change         regex replace
  delete         delete visible lines

Language-specific:
  declarations   search for class, function or struct definitions by name.
  imports        show include, import or use, depending on language.


Safety:
  explain        shows how your commands were interpreted or ignored.
  writes         use -O only after verifying output and preferably use on version-controlled files.

Examples:
  rxedit file 'all::TODO' 'more::print::i'

### Final Example with sample output:

`rxedit pysample.py declarations d::add::b less::__init__ more::print less::history::i`

````
000005 def do_init(func):
000009 class Calculator:
000017     def add(self, a: int, b: int) -> int:
000018         """Add two numbers  ."""
000019         result_add = a + b
000021         return result_add
000023     def multiply(self, a: int,
000024                 b: int) -> int:
000037             print(f"  {entry}")
````

