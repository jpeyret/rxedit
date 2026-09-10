//! Shared types and helpers used by many command modules.
//! This file holds app state, line metadata, and parsing results.
//! It also re-exports telemetry and command qualifiers used across the project.
//! Keep this file as the common layer between commands and utilities.
//! It helps the code stay consistent without repeating core structures.

use core::fmt;
use std::collections::HashSet;
use std::sync::RwLock;

/// Re-exported telemetry types and helpers.
pub use crate::telemetry::{
    NoGrammarNotification, TelemetryEvent, append_telemetry, clear_telemetry, request_explain,
    take_explain_requests, telemetry_events,
};
use crate::{constants::DEBUGGING, utilities::parse_lines_payload};
use once_cell::sync::Lazy;

/// Re-exported base command and line-state types.
pub use crate::base::{
    CheckCondition, CheckConditions, DeclarationsCommandQualifier, GrepCommandQualifier,
    LineStatus, MetaInfo,
};

#[derive(Debug, Clone, Default)]
/// Global runtime configuration shared by commands.
pub struct AppConfig {
    /// Active source file path.
    pub file_path: String,
    /// Whether to print spacing between blocks.
    pub spacer: bool,
    /// Whether to show line numbers.
    pub line_number: bool,
    /// Whether grep shortcodes are enabled.
    pub grep_shortcodes: bool,
    /// Whether verbose output is enabled.
    pub verbose: bool,
    /// Whether debug mode is enabled.
    pub debug: bool,
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = format!(
            "Global configuration:\n  space between blocks={}, show line numbers={}, grep shortcode substitution:{}\n  file={}\n",
            self.spacer, self.line_number, self.grep_shortcodes, self.file_path
        );
        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone, Default)]
/// Mutable app state wrapped by a global lock.
pub struct AppState {
    /// Current global configuration.
    pub config: AppConfig,
}

static APP_STATE: Lazy<RwLock<AppState>> = Lazy::new(|| RwLock::new(AppState::default()));

/// Returns a clone of the current global config.
pub fn get_global_config() -> AppConfig {
    APP_STATE
        .read()
        .expect("global app state lock poisoned")
        .config
        .clone()
}

/// Replaces the current global config.
pub fn set_global_config(config: AppConfig) {
    let mut state = APP_STATE.write().expect("global app state lock poisoned");
    state.config = config;
}

/// Mutates the global config in place via callback.
pub fn with_global_config<F>(f: F)
where
    F: FnOnce(&mut AppConfig),
{
    let mut state = APP_STATE.write().expect("global app state lock poisoned");
    f(&mut state.config);
}

/// Resets global state and telemetry for tests.
pub fn reset_global_state_for_tests() {
    let mut state = APP_STATE.write().expect("global app state lock poisoned");
    *state = AppState::default();
    clear_telemetry();
}

pub use crate::language_api::{Parsed, ParsedDeclarationsData, ParsedImportsData};

#[derive(Debug)]
/// Parsed search command seed before execution.
pub struct SearchSeed {
    /// Search strategy to execute.
    pub searcher: crate::Searcher,
    /// Optional parsed grep qualifier.
    pub qualifier: Option<GrepCommandQualifier>,
    /// Unused command tail content.
    pub unused: String,
}

#[derive(Debug)]
/// Union of command qualifier kinds.
pub enum CommandQualifier {
    /// Qualifier for grep-like commands.
    GrepQualifier(GrepCommandQualifier),
    /// Qualifier for declarations commands.
    DeclareQualifier(DeclarationsCommandQualifier),
}

impl fmt::Display for LineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vis = if self.visible { "💡" } else { " " };
        write!(
            f,
            "{:06} {vis} {:.40}: key={}",
            self.line_number, self.line, self.meta.key
        )
    }
}

/// Debug-prints a value when debug mode is enabled.
pub fn dbg_<T: std::fmt::Display>(hdr: &str, v: &T) {
    if !*DEBUGGING {
        return;
    }
    eprintln!("{hdr} {:.200}", v);
}

// not sure why &[T] is causing problems all over here, but this is just a debugging function, ignore it.
/// Debug-prints index values when debug mode is enabled.
#[allow(clippy::ptr_arg)]
pub fn dbg_indices<T: std::fmt::Display>(hdr: &str, indices: &Vec<T>) {
    if !*DEBUGGING {
        return;
    }
    let tmp: Vec<String> = indices.iter().map(|idx| format!("{}", idx)).collect();
    eprintln!("{hdr} {:.200}", tmp.join(", "))
}

impl GetLine for LineStatus {
    fn line(&self) -> &str {
        &self.line
    }
}


/// Result of a stop-or-add line evaluation.
pub enum ConditionResult {
    /// Stop scanning without adding line.
    Stop, // stop
    /// Add line and continue scanning.
    True, // add to hits
    /// Skip line and continue scanning.
    False, // dont add to hits - for example, when seeking an Owner
    /// Add line and then stop scanning.
    StopAndTrue,
}

/// Interface for line evaluators that can stop traversal.
pub trait StopOrAddChecker {
    // more complicated:  stop when you reach a certain point, but you may or may not add line
    /// Evaluates one line and returns the traversal action.
    fn is_match(&mut self, pos: i32, line: &LineStatus) -> ConditionResult;
}

/// Interface for observing line traversal state.
pub trait StateWatcher {
    /// Consumes one line event.
    fn feed(&mut self, pos: i32, line: &LineStatus);
}

/// Trait for types that expose line text.
pub trait GetLine {
    /// Returns the underlying line text.
    fn line(&self) -> &str;
}

/// updates LineStatus.meta.key - a marker which can be used to constrain command scope
pub fn update_meta_key(
    lines: Vec<LineStatus>,
    context_indices: &HashSet<usize>,
    key: &str,
) -> Vec<LineStatus> {
    lines
        .into_iter()
        .enumerate()
        .map(|(idx, mut line_status)| {
            if context_indices.contains(&idx) {
                line_status.meta.key = key.to_string();
            }
            line_status
        })
        .collect()
}

///set LineStatus.visible=true at positions specified by context_indices
pub fn set_visibles(lines: Vec<LineStatus>, context_indices: &HashSet<usize>) -> Vec<LineStatus> {
    lines
        .into_iter()
        .enumerate()
        .map(|(idx, mut line_status)| {
            if context_indices.contains(&idx) {
                line_status.visible = true;
            }
            line_status
        })
        .collect()
}

/// return number of indented whitespaces for line
/// return -1 if the line is all whitespace
pub fn calc_indent(line: &str) -> i32 {
    let len_ = line.len();
    let len_ltrim = line.trim_start().len();
    if len_ltrim > 0 {
        (len_ - len_ltrim) as i32
    } else {
        -1
    }
}

//     pub conds: Vec<Box<dyn CheckCondition>>,

#[derive(Debug, Clone)]
/// Condition that matches specific line numbers or ranges.
pub struct CheckLineRange {
    /// Inclusive upper bound for accepted line numbers.
    pub until_: usize,
    /// Explicit line numbers that are accepted.
    pub wanted: HashSet<usize>,
    /// Inclusive lower bound for accepted line numbers.
    pub from_: usize,
}

impl CheckLineRange {
    /// Builds a line-range condition from payload text.
    pub fn new(where_linenum: String) -> Self {
        let (until_, wanted, from_) = parse_lines_payload(&where_linenum);
        CheckLineRange {
            until_,
            wanted,
            from_,
        }
    }
}

impl Default for CheckLineRange {
    fn default() -> Self {
        CheckLineRange {
            until_: 0_usize,
            from_: 9_999_999_usize,
            wanted: HashSet::new(),
        }
    }
}

impl CheckCondition for CheckLineRange {
    fn check(&self, line: &LineStatus) -> bool {
        (line.line_number <= self.until_)
            || self.wanted.contains(&line.line_number)
            || (line.line_number >= self.from_)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
/// Condition that matches lines by metadata key.
pub struct CheckKey {
    /// Required metadata key value.
    pub required_key: String,
}

impl CheckKey {
    /// Creates a key-matching condition.
    pub fn new(key: String) -> Self {
        CheckKey { required_key: key }
    }
}

impl CheckCondition for CheckKey {
    fn check(&self, line: &LineStatus) -> bool {
        line.meta.key == self.required_key
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{LazyLock, Mutex};

    use super::{
        AppConfig, LineStatus, MetaInfo, get_global_config, reset_global_state_for_tests,
        set_global_config,
    };

    static TEST_GLOBAL_CONFIG_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn line_status_meta_defaults() {
        let status = LineStatus::default();

        assert_eq!(status.meta.key, "");
        assert_eq!(status.meta.groupid, 0);
    }

    #[test]
    fn meta_info_defaults() {
        let meta = MetaInfo::default();

        assert_eq!(meta.key, "");
        assert_eq!(meta.groupid, 0);
    }

    #[test]
    fn global_config_defaults_are_available_without_init() {
        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        reset_global_state_for_tests();

        let config = get_global_config();
        assert!(!config.spacer);
        assert!(!config.line_number);
        assert!(!config.grep_shortcodes);
        assert!(!config.verbose);
        assert!(!config.debug);
        assert_eq!(config.file_path, "");
    }

    #[test]
    fn global_config_can_be_reset_for_tests() {
        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        reset_global_state_for_tests();

        set_global_config(AppConfig {
            file_path: "sample.rs".to_string(),
            spacer: true,
            line_number: true,
            grep_shortcodes: true,
            verbose: true,
            debug: true,
        });

        let changed = get_global_config();
        assert!(changed.spacer);
        assert!(changed.line_number);
        assert!(changed.grep_shortcodes);
        assert!(changed.verbose);
        assert!(changed.debug);
        assert_eq!(changed.file_path, "sample.rs");

        reset_global_state_for_tests();
        let reset = get_global_config();
        assert!(!reset.spacer);
        assert!(!reset.line_number);
        assert!(!reset.grep_shortcodes);
        assert!(!reset.verbose);
        assert!(!reset.debug);
        assert_eq!(reset.file_path, "");
    }
}
