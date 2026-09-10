# Command `less` - [l]ess

Hides matching lines.

### Example:  hide "print_"

`rxedit sample.rs 'a::fn.*_message' less::print_`

- `a::fn.*_message` only lines matching the regex `fn.*_message` are shown.
   - This regex is quoted because the shell may interpret special characters.
- `less::print_` hides lines that contain `print_`.

#### Output

```
fn make_message() -> String {
    fn always_pass_message(){
```


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


##### Remark:

The [A]fter / [B]efore / [C]ontext flags don't work for now.  They're there because the intent is to hide 
matching lines and then hide before/after lines.

