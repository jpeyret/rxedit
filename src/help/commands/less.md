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
