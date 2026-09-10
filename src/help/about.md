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
- `all`          show only matching lines
- `more`         show more matching lines
- `less`         hide matching lines
- `invert`       invert visibility
- `lines`        displays lines within ranges, similar to a posix head or tail

#### Editing commands:
- `delete`       delete visible, matching, lines
- `change`       regex replacement on visible lines.  Supports the full Rust regex syntax.
- `prepend`      add a line before the matching line
- `append`       add a line after the matching line

#### Language-dependent commands (will become no-ops if the language is not supported)

(currently supports: `.py` and `.rs` extensions)

- `declarations`  look for class, struct or function names that match.
- `imports`       look for include, import or use lines that match the name

#### Utility commands:

- `macro`        loads commands from a given file, allowing predefinition of search and exclusion patterns
- `explain`      stderr what rxedit has understood, and rejected, from user input.


#### Flags (not all commands support all flags, see their individual help)

- `i` case insensitive.
- `F` fixed string, meaning that "string contains" will be used, rather than a regex match.
- `A10`, `B2`, `C1` - similar to using `grep -A 10 -B 2`...
- `o` owner flag - shows the function or class signature to which a matching lines belongs
- `b` body flag - only avaiable on `declarations` commands, shows all lines for a function or class.
- `wt=fs`. For `declarations` only:  filters the types of definitions we are interested in.  On Rust, `fs`
   would match `fn` or `struct`.
- `u` user confirmation, match by match, on `delete` or `change` commands.
- `sk=debug.`  Sets `key=debug` on matching lines.  This can be used to constrain further commands using `wk`.
- `wk=debug.`  In addition to any match conditions, the command will only affect lines with `key=debug` set. 
- `ln=1-10.` In addition to any match conditions the command will only affect the first 10 lines.

Flags are designed to be mixed freely together in any order:  

`more::debug::A2sk=devonly.i` will recognize `A2`, case insensitive and setting `key=devonly` on matching lines.


#### Error handling

Unknown commands, malformed flags, and unsupported language-specific operations are treated as no-ops 
to allow for quicker experimentation.

Use `explain` to see how rxedit interpreted your command sequence.  And fine-tune your visible lines
before adding editing commands,


### Command line syntax:
