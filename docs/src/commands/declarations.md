# Command `declarations` - [d]eclarations  (supported languages only)

Shows declarations/definitions, for supported languages and allows searching by name,
as well as parent-level filtering.

- `::o` the "owner" flag can show the declaration it belongs to, for example the class a
matched method belongs to.
- `::b` the "body" flag shows the lines that belong to the declaration.

#### Example:  declarations matching `init`.

`rxedit pysample.py d::__init__::`

#### Output:

```
def do_init(func):
    def __init__(self, name: str):
```

#### show the owner and filter only function declarations with `wt=f.`

`rxedit sample.rs d::message::wt=f.o`

#### Output:

```
fn make_message() -> String {
fn print_message(message: &str) {
mod tests {
    fn always_pass_message(){
```

#### same as above, but filter out functions that belong to parents matching `test`

`rxedit sample.rs d::--test/message::wt=f.o`

#### Output:

```
fn make_message() -> String {
fn print_message(message: &str) {
```

You could also have a positive parent test using `d::test/message::`.


## Declarations

|Name                  | Format               | Example                       | Description|
|----------------------| ---------------------| ------------------------------| -----------|
|show_body             | b                    | d::::b                        | body of the definition|
|show_owner            | o                    | ::o                           | show "owner" of the line|
|where_type            | wt=([a-z]+)(?:\.\|$) |  ::wt=f. (shows only functions|  only show some types of declarations|
|insensitive           | i                    | ::i                           | case-insentive search|
|after                 | A(\d+)               | ::A16                         | Show NUM lines after each match.|
|before                | B(\d+)               | ::B5                          | Show NUM lines before each match.|
|context               | C(\d+)               | ::C5                          | Show NUM lines before and after each match.|
|fixed_string          | F                    | ::F                           | treat search as regular string, not a regex|
|use_grep_shortcodes   | g                    | ::g                           | expands shortcodes (`..`→`.*`, `~~s.`→` +`)|
|no_use_grep_shortcodes| G                    | ::G                           | does not expand grep shortcodes|
|show_owner            | o                    | ::o                           | show "owner" of the line|
|set key               | sk=([a-z]+)(?:\.\|$) |  ::sk=important.              |  set key to a value that can be used with wk=<key>. in later commands|
|where_key             | wk=([a-z]+)(?:\.\|$) |  ::wk=mykey.                  |  puts an extra criteria to other commands|
|where_linenum         | wl=([0-9,-]+)(?:\.\|$|  ::wl=-5,22,99.               |  limit line number range|

### Notes:

Declarations are available when the file extension is recognized as a supported language.  If not,
this will become a [`noop`](./noop.md).  

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


### SEE ALSO 

  [`shortcodes`](../shortcodes.md), [chaining](../chaining.md), [`languages`](../languages.md), [`noop`](./noop.md)

