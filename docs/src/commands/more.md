# Command more - [m]ore

Shows matching lines.

### Example: show 'message'

`rxedit sample.rs more::message::o`

- `more::message` shows lines matching `message`. 
- `::o`           for a supported language, the "owner" (class, function, struct, etc...) gets shown

#### Output

```
fn make_message() -> String {
fn print_message(message: &str) {
    println!("{message}");
fn main() {
    let message = make_message();
    print_message(&message);
```

#### More examples:

- `rxedit sample.rs  m::message::oFi`  same thing, but match as a fixed, case-insentive string
- `rxedit sample.rs  m::message::A2`   like `grep -A 2 message sample.rs`
- `rxedit sample.rs 'm::print..\(::g'` uses `..` shortcode to show any line matching `print*(`


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

