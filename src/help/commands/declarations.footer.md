
### Notes:

Declarations are available when the file extension is recognized as a supported language.  If not,
this will become a `noop`.  

Source parsing is done via tree-sitter which provides contextual knowledge such as:

- whether a declaration is nested under a parent. 
- which lines of code "belong" to a given declaration, giving the same capabilities as a folding editor.
- multi-line function signatures are displayed as a block
- the ability to filter by parent name, using `!` or `--` for negation and `/` for level.

#### Search matching

Note that only the name of the declaration is matched against.

Since names are alphanumeric this means grep shortcodes are limited to OR (using `|` or `,`) and WILDCARD (`..`).
But also that the searching is also clever enough to support both positive and negative matching, 
so `'d::split--regex,nega'` looks for names with `split` but not `regex` or `nega`.

Standard search  flags like `A2` or `sk=mykey.` are also supported by `declarations`.

#### where type, `wt=xyz` flag

Each language also supports filtering by declaration type using the `wt=` flag.  
`::wt=fs` on rust means search only `fn` and `struct` names, 
whereas `d::::wt=c` would show only Python `class` definitions.


### SEE ALSO --help for 

  `shortcodes`, `chaining`, `languages`, `noop`
