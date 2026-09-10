# Command All - [a]ll

Hides everything, then shows matching lines.

#### show lines matching `Greet`, case-insentive:

(equivalent to `grep -i Greet sample.rs`)

`rxedit sample.rs a::Greet::i`

#### Output

```
trait Greeter {
	fn greet(&self, name: &str) -> String;
impl Greeter for HelloApp {
	fn greet(&self, name: &str) -> String {
	app.greet("world")
```

#### More examples:

- `rxedit sample.rs  a::Greet::Fi` would do the same thing, but match as a fixed, case-insentive, string
- `rxedit sample.rs  a::Greeter::A1` will work like `grep -A 1 Greeter sample.rs`


## Search flags

|Name                  | Format               | Example        | Description|
|----------------------| ---------------------| ---------------| -----------|
|insensitive           | i                    | ::i            | case-insentive search|
|after                 | A(\d+)               | ::A16          | Show NUM lines after each match.|
|before                | B(\d+)               | ::B5           | Show NUM lines before each match.|
|context               | C(\d+)               | ::C5           | Show NUM lines before and after each match.|
|fixed_string          | F                    | ::F            | treat search as regular string, not a regex|
|use_grep_shortcodes   | g                    | ::g            | expands shortcodes (`..`→`.*`, `~~s.`→` +`)|
|no_use_grep_shortcodes| G                    | ::G            | does not expand grep shortcodes|
|show_owner            | o                    | ::o            | show "owner" of the line|
|set key               | sk=([a-z]+)(?:\.\|$) |  ::sk=important|  set key to a value that can be used with wk=<key>. in later commands|
|where_key             | wk=([a-z]+)(?:\.\|$) |  ::wk=mykey.   |  puts an extra criteria to other commands|
|where_linenum         | wl=([0-9,-]+)(?:\.\|$|  ::wl=-5,22,99.|  limit line number range|
### SEE ALSO 

  [`shortcodes`](../shortcodes.md), [chaining](../chaining.md), [`noop`](./noop.md)

