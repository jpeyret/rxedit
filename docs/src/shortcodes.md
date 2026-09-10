
# Grep macro shortcodes.
Shortcodes can be applied to all regex searches, using the `-g` flag to turn it on (`-G` turns shortcodes off ).
They facilitate some common regex use cases and avoid clashing with shell substitutions.

##### Examples:  
- `rxedit sample.rs 'm::impl.*for'` can be replaced with `rxedit -g sample.rs m::impl..for`.  Note that quotes aren't needed anymore.
- `rxedit sample.rs 'm::fn|struct'` with `rxedit sample.rs -g m::fn~~o.struct `

##### Available shortcuts:

|Key              | Macro   | Expa|sion  Description|
|-----------------| --------| ----| -----------|
|wildcard         | ..      | .*  | matches any string|
|spaces           | ~~s.    |  +  | matches one or more space character|
|or               | ~~o.    | \|  |  standard grep OR/pipe a\|b|
|quotes           | ~~q.    | ['"]| matches either a single or double quote character|
|less_than        | ~~lt.   | <   | matches `<`|
|left_parenthesis | ~~pl.   | \(  | matches `(`|
|right_parenthesis| ~~pr.   | \)  | matches `)`|
|field_separator  | ~~sep.  | ::  | matches `::`|
|colon            | ~~colon.| :   | matches `:`|

##### Remarks:

- Searching on identifier names, via the `declare` command only supports the `wildcard` and `or` shortcodes because identifiers are basically alphanumeric only.
  So `declare::foo..bar>::` is valid but `declare::foo~~s.bar>::` will remain unexpanded.
- They don't work with fixed string matching, only with regexes.  `m::..::F` willl match only a literal `..` while `m::..::` will match everything.

