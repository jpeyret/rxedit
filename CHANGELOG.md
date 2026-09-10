# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
