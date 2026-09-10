
##### Remarks:

- Searching on identifier names, via the `declare` command only supports the `wildcard` and `or` shortcodes because identifiers are basically alphanumeric only.
  So `declare::foo..bar>::` is valid but `declare::foo~~s.bar>::` will remain unexpanded.
- They don't work with fixed string matching, only with regexes.  `m::..::F` willl match only a literal `..` while `m::..::` will match everything.
