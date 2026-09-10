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
