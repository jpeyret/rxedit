# Command Delete - [del]ete

Deletes visible lines matching pattern.

## Example delete visible lines containing "debug"

`rxedit sample.rs a::debug delete:debug`


## Details

- `delete` removes lines from the output entirely (not just hiding them).
- use [`less`](./less.md) or set key / where key (`sk=`,`wk=`) to constraint the scope.


## delete flags

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
### Notes about commands that modify the text

- Nothing gets saved until you use the `-O` modify in place flag or the `-o <some path>` output flag.
- Those commands only apply to matching visible lines, unlike say a `sed` insert or delete.
- `append`,`prepend`,`delete` will throw off the line numbers used in `where line` flags
- When modifying a source code file, the result may not be syntaxically correct, so be careful.
  - rxedit is designed on the expectation that *modification* commands will be used on version-controlled files.
  - but you could use the `-o myfile.bak` flag to save and diff your changes first.
- Use [`explain`](./explain.md) to make sure all previous commands to show/hide lines went as expected.
- Modification commands support the `::u` user confirmation flag, which will prompt the user for confirmation.  
  - Use of that flag will, by design, trigger an immediate error in batch mode.
- rxedit will "follow" symlinks so it will affect the actual file if you modify it.

### SEE ALSO 

  [`shortcodes`](../shortcodes.md), [chaining](../chaining.md), [`noop`](./noop.md), [flags](../flags.md)

