### Notes about commands that modify the text

- Nothing gets saved until you use the `-O` modify in place flag or the `-o <some path>` output flag.
- Those commands only apply to matching visible lines, unlike say a `sed` insert or delete.
- `append`,`prepend`,`delete` will throw off the line numbers used in `where line` flags
- When modifying a source code file, the result may not be syntaxically correct, so be careful.
  - rxedit is designed on the expectation that *modification* commands will be used on version-controlled files.
  - but you could use the `-o myfile.bak` flag to save and diff your changes first.
- Use `explain` to make sure all previous commands to show/hide lines went as expected.
- Modification commands support the `::u` user confirmation flag, which will prompt the user for confirmation.  
  - Use of that flag will, by design, trigger an immediate error in batch mode.
- rxedit will "follow" symlinks so it will affect the actual file if you modify it.

### SEE ALSO --help for 

  `shortcodes`, `chaining`, `noop`, `flags`
