# Command invert

Flips visibility for every line.  Anything that was visible becomes invisible 
and vice versa.  Useful to extract matching lines for a report.  First show
everything you want, then `invert`, then `delete`.


### Example: extract the public API from a rust file.

`rxedit lib.rs d::--tests 'and::^pub ' invert delete all`

- `d::-tests/` shows all declarations, not under `mod tests`
- `'and::^pub '` keep only those lines with `pub` keyword
- `invert` flips visibility.
- `delete::` removes currently visible lines, i.e. not a public definition
- [`all`](./all.md) show all lines.  (In practice use something like `-o signatures.out` to write to a report file)

#### Output:

```
pub trait CommandActions {
pub enum Command {
pub fn make_command(arg: &str) -> Command {
pub fn commands_factory(args: &[String]) -> Vec<Command> {
```


## Invert flags:

|Name| Forma|  Exampl|  Description|
|----| -----| -------| -----------|

### Notes:

`invert` has no flags or payload

### SEE ALSO

  [chaining](../chaining.md)

