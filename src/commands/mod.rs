//! all CommandActions implementors are grouped under this directory.
/// Shows all lines or matching lines.
pub mod all;
/// Combines current visibility with a search filter.
pub mod and;
/// Prepends or appends content around matched lines.
pub mod appendprepend;
/// Replaces text in visible matched lines.
pub mod change;
/// Shared command helpers and utilities.
pub mod core;
/// Grammar-aware declarations command.
pub mod declarations;
/// Deletes visible matched lines.
pub mod delete;
/// Explains command parsing and effects.
pub mod explain;
/// Grammar-aware imports command.
pub mod imports;
/// Inverts line visibility state.
pub mod invert;
/// Hides lines matching a pattern.
pub mod less;
/// Selects lines by line-range expressions.
pub mod lines;
/// Loads and expands macro command files.
pub mod macros;
/// Shows lines matching a pattern.
pub mod more;
/// No-op fallback command.
pub mod noop;
/// Common imports used by command modules.
pub mod prelude;
/// Shared search command implementation.
pub mod search;
