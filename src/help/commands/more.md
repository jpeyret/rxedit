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
