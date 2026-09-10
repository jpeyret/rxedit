# Command Change - [c]hange

Change contents visible lines matching pattern, similar to `sed`.


## Example: remove "//v2help" comments.

First step is to make lines visible, via ` rxedit help_utilities.rs m:://v2help::o`


```
pub fn print_topic_help(topic: &str, config_path: &str) -> Result<(), String> {
        "explain" => { //v2help
        "all" => { //v2help
        "and" => { //v2help
  ...(more lines)...
```

Then change them:

`rxedit help_utilities.rs m:://v2help::o change:://v2help::::`

The `-O ` flag will modify the file in place, when you use it. 

```
pub fn print_topic_help(topic: &str, config_path: &str) -> Result<(), String> {
        "explain" => { 
        "all" => { 
        "and" => { 
  ...(more lines)...
```

### Example: comment out source code 

`change` really ends up calling Rust's Regex crate under the cover, so things that are 
syntaxically correct for that crate work here as well.

`rxedit -n pysample.py m::print::osk=tocomment`

Show all lines with "print" and their owner.  Set `key=tocomment`.

#### Output pass 1 

```
000031     def show_history(self) -> None:
000035         print(f"\n{self.name}'s History:")
000037             print(f"  {entry}")
```

`rxedit -n pysample.py m::print::osk=tocomment less::::wl=35 'c::([\s]*)([^\s].*$)::$1#$2::wk=tocomment'`

Remove line 35 from scope and then use regex implicit group names to comment things out.

Note how "show_history" was not affected, because `::wk=tocomment` filtered it out on the `change`

#### Output pass 2 

```
000031     def show_history(self) -> None:
000037             #print(f"  {entry}")
```

Since that "comment out" is complicated to remember, you can save it in a `macro` file instead.
(Be mindful about setting `sk=tocomment` if you keep `wk=tocomment` in the macro.)

`rxedit -n pysample.py m::print::osk=tocomment less::::wl=35 macro::~/mycommentoutforrust.rxi`
