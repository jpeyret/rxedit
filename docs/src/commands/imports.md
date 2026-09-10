# Command `imports` - [i]imports (supported languages only)

Find imports/uses/includes that match the search term.

#### Example: search for imports that match `co`, but not `std` or `regex`.

`rxedit utilities.rs i::co--std,regex`

#### Output:

```
use crate::common::{CheckLineRange, LineStatus};
use crate::constants::{DEBUGGING, RE_SPLIT_NEGATIVES_NAMES};
    use crate::common::ConditionResult;
    use crate::common::GetLine;
    use crate::common::StopOrAddChecker;
    use crate::constants::Direction;
    use crate::common::LineStatus;
    use crate::constants::Direction;
```

#### Example: search for imports "far in the file"

Another example if we want to look for imports that are past the reasonable start of the file, line 100 in 
the example below.
Sometimes however, with Python, this is done to avoid circular import dependencies, so we'll use the `::o`
"show owner" flag as well.

`rxedit utils.py i::::wl=100-.o`


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

### Notes:

Imports are available when the file extension is recognized as a supported language.  If not,
this will become a [`noop`](./noop.md).  

The search mechanism behaves like [`declarations`](./declarations.md) name matching, because
the names are basically expected to be alphanumeric.

Name matching is a bit complicated (AND A WORK IN PROGRESS).

Imports names are matched against each level but the last.  
That's because this is matching against the modules imported rather than what is imported from a module.

So `i::std` would match `use std::fs;` but `i::fs` would not because `fs` is the last bit.

### SEE ALSO 

  [`shortcodes`](../shortcodes.md), [chaining](../chaining.md), [`languages`](../languages.md), [`noop`](./noop.md)

