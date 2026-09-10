
### Notes:

Imports are available when the file extension is recognized as a supported language.  If not,
this will become a `noop`.  

The search mechanism behaves like `declarations` name matching, because
the names are basically expected to be alphanumeric.

Name matching is a bit complicated (AND A WORK IN PROGRESS).

Imports names are matched against each level but the last.  
That's because this is matching against the modules imported rather than what is imported from a module.

So `i::std` would match `use std::fs;` but `i::fs` would not because `fs` is the last bit.

### SEE ALSO --help for 

  `shortcodes`, `chaining`, `languages`, `noop`
