//! defines low-level components used by other files

use crate::constants::FlagDef;
use core::fmt;
use regex::Regex;
use std::any::Any;

/// checks if a source line matches a generic condition
pub trait CheckCondition: fmt::Debug {
    /// Returns true when the condition matches the given line.
    fn check(&self, line: &LineStatus) -> bool;
    /// Supports downcasting to concrete condition types.
    fn as_any(&self) -> &dyn Any;
}

impl fmt::Debug for CheckConditions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CheckConditions({})", self.conds.len())
    }
}

/// tracks information about a source line
#[derive(Clone, Default, Debug)]
pub struct MetaInfo {
    /// tracks if that line has been "flagged" with a key
    pub key: String,
    /// Optional grouping id used by commands.
    pub groupid: i32,
}

/// state of one source text line.
#[derive(Clone, Default, Debug)]
pub struct LineStatus {
    /// 0-based source line number.  Note:  user-facing is 1-based, internal is 0-based
    pub line_number: usize,
    /// Whether the line is currently visible.
    pub visible: bool,
    /// The current text of the line.
    pub line: String,
    /// Extra metadata associated with the line.
    pub meta: MetaInfo,
}

/// allows combination of multiple generic conditions
pub struct CheckConditions {
    /// Registered conditions evaluated for each line.
    pub conds: Vec<Box<dyn CheckCondition>>,
}

/// tracks compile-time constants about a given Command:
/// flags it supports, the name and whether it can mutate
/// source code lines (only change/prepend/append/delete do that)
#[derive(Debug, Default)]
pub struct CommandDefinition {
    /// Flags accepted by the command.
    pub flags: &'static [FlagDef],
    /// Canonical command name.
    pub name: &'static str,
    /// True when command can mutate line text.
    pub mutates_line_text: bool,
}

impl Default for CheckConditions {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckConditions {
    /// Creates an empty condition set.
    pub fn new() -> CheckConditions {
        CheckConditions { conds: Vec::new() }
    }

    /// Adds one condition to the condition set.
    pub fn add_condition<C: CheckCondition + 'static>(&mut self, checker: C) {
        self.conds.push(Box::new(checker));
    }

    /// do all conditions match for this line?
    pub fn check(&self, line: &LineStatus) -> bool {
        self.conds.iter().all(|checker| checker.check(line))
    }
}

/// tracks user intent for a search type command.
#[derive(Debug)]
pub struct GrepCommandQualifier {
    /// Number of lines to include after a match.
    pub after: i32,
    /// Number of lines to include before a match.
    pub before: i32,
    /// Whether to apply grep shortcode expansion.
    pub preformat: bool,
    /// Whether to treat pattern as fixed text.
    pub fixed_string: bool,
    /// Regex flags passed to regex compilation.
    pub regex_flags: Option<String>,
    /// If `Some(key)`, set `meta.key` on every line that matches the search pattern.
    pub set_key: Option<String>,
    /// Whether to display owner information.
    pub showowner: bool,
    /// Whether command requires user confirmation.
    pub user_confirmation: bool,
    /// unrecognized stores command contents that were not understood.  `explain` shows them.
    pub unrecognized: String,
    /// Extra where-conditions applied after searching.
    pub conditions_checker: CheckConditions,
}

/// CDeclaration can place constraints on the parents that are being matched against
/// `d::--tests/::` for example will not match any items belonging to a parent matching `tests`
#[derive(Debug)]
pub enum DeclarationsParentMatcher {
    /// Fixed-string parent matching rules.
    Fixed {
        /// Parent tokens that must match.
        positive: Vec<String>,
        /// Parent tokens that must not match.
        negative: Vec<String>,
        /// Whether matching ignores case.
        case_insensitive: bool,
    },
    /// Regex parent matching rules.
    Regex {
        /// Regex that must match.
        positive: Regex,
        /// Optional regex that must not match.
        negative: Option<Regex>,
        /// Whether matching ignores case.
        case_insensitive: bool,
    },
}

/// tracks user intent for a CDeclare command
#[derive(Debug)]
pub struct DeclarationsCommandQualifier {
    /// Shared search qualifier options.
    pub searchqualifier: GrepCommandQualifier,
    /// File extension used for grammar lookup.
    pub extension: String,
    /// Whether declaration docs should be shown.
    pub showdoc: bool,
    /// Whether declaration bodies should be shown.
    pub showbody: bool,
    /// unrecognized stores command contents that were not understood.  `explain` shows them.
    pub unrecognized: String,
    /// Optional parent filters from command input.
    pub parents: Option<Vec<String>>,
    /// Compiled parent matchers.
    pub parent_matchers: Option<Vec<DeclarationsParentMatcher>>,
    /// Optional declaration type filter.
    pub where_type: Option<String>,
}
