# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2024-10-09

-     added commands `appendfile` / `prependfile which  include a file's contents at the insertion point

-     added command line flag -R, --report-file:  write only visibles to file.

-     added command line flag --no-empty-write:  don't write outputs with -o, -O or -R flags if empty

-     enabling Git Action CI checks via rust.yml

-     fixed `declarations` not finding Python declarations with an inline comment

-     generate_mdbook.sh adds the GitHub badges to the top of README.md, via `prependfile`.

-     removed some of the shorter command abbreviations like 'del' for `delete`

## [0.1.2] - 2024-09-25

### Changed

- Change the default search pattern to match empty lines.
- macro commands now receive their file as a normal path, in the next argument.  i.e. `rxedit target macro::<path_to_macro>` becomes `rxedit target macro <path_to_macro>` allowing for tab completion.  A missing macro file now stops execution, much like a missing target as first argument.
- the line number criteria syntax has been changed to `<from>..<to>`, instead of `<from>-<to>`.  Negative offsets, from the end, are supported.  `rxedit target lines::..10,-5..` is now roughly equivalent to concatenating `head -10 target` and `tail -5 target`.  grep-style before/after flags are also supported, with `rxedit target lines::37C10` now printing lines 27-47.
- The field separator in the flags is now `/` rather than `.`.  So `rxedit target more::def ::wl=1..20/i` rather than ``more::def ::wl=1..20.i`.  This is equivalent to `head -20 target | grep -i 'def '`.

### Fixed

- Deletion no longer leaves blank lines behind.

## [0.1.1] - 2024-09-03

### Added
- Initial public release of rxedit
- Folding batch editor for source code discovery and editing
- Support for regex-based pattern matching with tree-sitter awareness
- Composable editing commands: change, delete, insert, prepend, append, declare
- Command-line interface with comprehensive help and examples
- Integration tests for CLI functionality
- mdBook-based documentation
- Code coverage reporting via `cargo llvm-cov`

### Features
- Regex matching with flags (case-insensitive, fixed-string)
- Tree-sitter support for Rust and Python
- Before/after context display
- Visibility control (show, hide, invert, more, less)
- Large-scale structured text refactoring
- Scriptable via command files

### Changed
- Project versioned at 0.1.1 for initial release

### Fixed
- Clippy warnings resolved
- Code formatting verified
