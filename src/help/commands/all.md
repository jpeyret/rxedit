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
