//! lib.rs
#![warn(missing_docs)]
#![warn(dead_code)]
pub mod base;
/// Shared helpers for reading and creating the user's config.toml file.
pub mod config_utilities;
// commands
pub mod commands;
pub mod common;
pub mod constants;
pub mod flag_utilities;
pub mod formatters;
pub mod language_api;
pub mod languages;
pub mod loader_for_constants;
pub mod telemetry;
pub mod treesitterparser;
pub mod user_messages;
pub mod utilities;
use crate::common::CommandQualifier;
use crate::constants::command_prefix;
use crate::constants::grep_shortcodes::GrepShortCodeDef;
pub use commands::all::CAll;
pub use commands::and::CAnd;
pub use commands::appendprepend::CInserter;
pub use commands::change::CChange;
pub use commands::declarations::CDeclarations;
pub use commands::delete::CDelete;
pub use commands::explain::CExplain;
pub use commands::imports::CImports;
pub use commands::invert::CInvert;
pub use commands::less::CLess;
pub use commands::lines::CLines;
pub use commands::macros::CMacro;
pub use commands::more::CMore;
pub use commands::noop::CNoop;
pub use commands::search::Search;
pub use commands::search::Searcher;
use enum_dispatch::enum_dispatch;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;

use crate::base::CommandDefinition;
use crate::commands::search::{ContainsKeepSkip, ContainsSearcher, RegexKeepSkip, RegexSearcher};
use crate::common::{GrepCommandQualifier, LineStatus, TelemetryEvent, append_telemetry};

use crate::constants as c;
use crate::constants::FlagDef;

#[derive(Debug, Clone, Copy)]
enum CommandVariant {
    CAll,
    CMore,
    CAnd,
    CLess,
    CDelete,
}

pub(crate) struct ChangeSeed {
    pub replacer: ChangeReplacer,
    pub qualifier: Option<GrepCommandQualifier>,
}

pub(crate) fn new_searcher(
    arg0: &str,
    skip_pattern: Option<&str>,
    flags: &str,
    allowed_flags: &'static [FlagDef],
    grep_shortcodes: &[GrepShortCodeDef],
) -> Result<common::SearchSeed, regex::Error> {
    let (qualifier, _unconsumed) = GrepCommandQualifier::build_with_flags(flags, allowed_flags);
    let pattern = preformat(arg0, &qualifier, grep_shortcodes);
    let skip_pattern = skip_pattern.map(|value| preformat(value, &qualifier, grep_shortcodes));

    let searcher = if qualifier.fixed_string {
        let mut case_insensitive: bool = false;
        if let Some(regex_flags) = qualifier.regex_flags.as_ref() {
            case_insensitive = regex_flags.contains(c::commandflags::CASE_INSENSITIVE.value);
        }
        match skip_pattern {
            Some(skip) => {
                Searcher::ContainsKeepSkip(ContainsKeepSkip::new(pattern, skip, case_insensitive))
            }
            None => Searcher::ContainsSearcher(ContainsSearcher::new(pattern, case_insensitive)),
        }
    } else {
        let regex_pattern = |value: String| {
            if let Some(regex_flags) = qualifier.regex_flags.as_ref() {
                format!("(?{}){}", regex_flags, value)
            } else {
                value
            }
        };

        match skip_pattern {
            Some(skip) => Searcher::RegexKeepSkip(RegexKeepSkip {
                rekeep: Regex::new(&regex_pattern(pattern))?,
                reskip: Regex::new(&regex_pattern(skip))?,
            }),
            None => Searcher::RegexSearcher(RegexSearcher {
                patre: Regex::new(&regex_pattern(pattern))?,
            }),
        }
    };

    Ok(common::SearchSeed {
        searcher,
        qualifier: Some(qualifier),
        unused: _unconsumed,
    })
}

#[derive(Debug)]
pub(crate) enum ChangeReplacer {
    Regex {
        patre: Regex,
        replace_with: String,
        global: bool,
        linefeed: bool,
    },
    Literal {
        pattern: String,
        replace_with: String,
        global: bool,
        linefeed: bool,
    },
}

impl ChangeReplacer {
    pub(crate) fn apply(&self, line: &str) -> String {
        match self {
            ChangeReplacer::Regex {
                patre,
                replace_with,
                global,
                linefeed: _,
            } => {
                if *global {
                    patre.replace_all(line, replace_with.as_str()).to_string()
                } else {
                    patre.replacen(line, 1, replace_with.as_str()).to_string()
                }
            }
            ChangeReplacer::Literal {
                pattern,
                replace_with,
                global,
                linefeed: _,
            } => {
                if *global {
                    line.replace(pattern, replace_with)
                } else {
                    line.replacen(pattern, replace_with, 1)
                }
            }
        }
    }
}

// use rxedit::constants::{DEBUGGING,commandflags};

pub(crate) fn new_replacer(pattern_arg: &str, replace_with: &str, flags: &str) -> ChangeSeed {
    let (qualifier, _unconsumed) =
        GrepCommandQualifier::build_with_flags(flags, c::commandflags::change_flags());

    let mut replace_with = replace_with.to_string();

    if *c::DEBUGGING {
        dbg!(&replace_with);
    }

    use unescaper::unescape;
    let pattern = preformat(pattern_arg, &qualifier, c::grep_shortcodes::searchfield());
    let global = flags.contains(c::commandflags::CHANGE_ALL.value);
    let case_insensitive = flags.contains(c::commandflags::CASE_INSENSITIVE.value);

    if !flags.contains(c::commandflags::RAW_STRING.value) {
        replace_with = unescape(&replace_with).unwrap();
    }

    const NL: &str = "\n";
    let linefeed = replace_with.contains(NL);

    if *c::DEBUGGING {
        dbg!(&linefeed, &replace_with);
    }

    let replacer = if qualifier.fixed_string && !case_insensitive {
        ChangeReplacer::Literal {
            pattern,
            replace_with: replace_with.to_string(),
            global,
            linefeed,
        }
    } else {
        let regex_pattern = if qualifier.fixed_string {
            regex::escape(&pattern)
        } else {
            pattern
        };
        ChangeReplacer::Regex {
            patre: RegexBuilder::new(&regex_pattern)
                .case_insensitive(case_insensitive)
                .build()
                .expect("invalid change regex pattern"),
            replace_with: replace_with.to_string(),
            global,
            linefeed,
        }
    };

    ChangeSeed {
        replacer,
        qualifier: Some(qualifier),
    }
}

#[enum_dispatch]
/// A CommandAction basically receives a `Vec<LineStatus>` which
/// are mostly a source code line and its visible boolean
/// and returns another.  Most CommandActions only manipulate
/// the visible boolean.
pub trait CommandActions {
    /// returns compile-time information about a command for introspection.
    fn get_constant_definition(&self) -> CommandDefinition;

    /// The core of rxedit.  Each Command receives a Vec of source code
    /// and decides to either show/hide or modify their content which
    /// is then passed on the next Command.
    /// the do-nothing version just returns the vec as is
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        lines
    }
}

fn search_vector(
    f_calculate_visibility: fn(bool, bool) -> bool,
    searcher: &Searcher,
    lines: Vec<LineStatus>,
    _variant: CommandVariant,
    qualifier: &GrepCommandQualifier,
) -> (Vec<usize>, Vec<LineStatus>) {
    let mut hit_positions = Vec::new();

    let result_lines = lines
        .into_iter()
        .enumerate()
        .map(|(idx, mut line_status)| {
            let hit = searcher.search(&line_status.line)
                && qualifier.conditions_checker.check(&line_status);
            if hit {
                hit_positions.push(idx);
                if let Some(ref key) = qualifier.set_key {
                    line_status.meta.key = key.clone();
                }
            }
            line_status.visible = f_calculate_visibility(line_status.visible, hit);
            line_status
        })
        .collect();
    (hit_positions, result_lines)
}

#[derive(Debug)]
#[allow(missing_docs)]
#[enum_dispatch(CommandActions)]
/// Polymorphic command wrapper used to dispatch to the appropriate CommandActions subtype.
pub enum Command {
    CLines(CLines), // show based on line ranges, similar `head` or `tail`
    /// Keeps visible only lines that satisfy all configured conditions.
    CAnd(CAnd),
    CMore(CMore),                 // toggle lines matching the pattern to visible
    CDeclarations(CDeclarations), // syntax-aware visibility command
    CImports(CImports),           // import-aware visibility command
    CLess(CLess),                 // hide lines matching the pattern
    CAll(CAll),       // display only lines matching the pattern given or all if no pattern
    CChange(CChange), // replace text in visible lines
    CDelete(CDelete), // delete visible lines matching the pattern
    CInserter(CInserter), // insert text before/after visible lines matching the pattern
    CInvert(CInvert), // flips visibility for all lines
    CExplain(CExplain), // emits telemetry debug output
    CNoop(CNoop),     // do-nothing pass through
    CMacro(CMacro),   // loads commands from file at location
}

impl Command {
    /// returns user-supplied arguments for the Command: its "Qualifier"
    pub fn qualifier(&self) -> Option<&CommandQualifier> {
        match self {
            Command::CAnd(c) => Some(&c.qualifier),
            Command::CMore(c) => Some(&c.qualifier),
            Command::CDeclarations(_c) => None,
            Command::CImports(c) => Some(&c.qualifier),
            Command::CLess(c) => Some(&c.qualifier),
            Command::CAll(c) => Some(&c.qualifier),
            Command::CChange(c) => c.qualifier.as_ref(),
            Command::CDelete(c) => Some(&c.qualifier),
            Command::CInserter(c) => c.qualifier.as_ref(),
            Command::CInvert(_) => None,
            Command::CExplain(_) => None,
            Command::CNoop(_) => None,
            Command::CLines(_c) => None,
            Command::CMacro(_) => None,
        }
    }

    /// Can this Command mutate line text, rather than just visibility and meta info?
    pub fn mutates_line_text(&self) -> bool {
        self.get_constant_definition().mutates_line_text
    }
}

fn command_source_for_generic_telemetry(command: &Command) -> Option<&'static str> {
    match command {
        Command::CAnd(_) => Some(command_prefix::AND),
        Command::CMore(_) => Some(command_prefix::MORE),
        Command::CImports(_) => Some(command_prefix::IMPORTS),
        Command::CLess(_) => Some(command_prefix::LESS),
        Command::CAll(_) => Some(command_prefix::ALL),
        Command::CChange(_) => Some(command_prefix::CHANGE),
        Command::CDelete(_) => Some(command_prefix::DELETE),
        Command::CInserter(c) => Some(c.source_prefix()),
        Command::CInvert(_) => Some(command_prefix::INVERT),
        Command::CExplain(_) => Some(command_prefix::EXPLAIN),
        Command::CDeclarations(_) | Command::CNoop(_) => None,
        Command::CLines(_) => Some(command_prefix::LINES),
        Command::CMacro(_) => todo!(),
    }
}

fn append_generic_command_telemetry(command: &Command, arg: &str, searcher_in: Option<&Searcher>) {
    let Some(source) = command_source_for_generic_telemetry(command) else {
        return;
    };

    match command {
        Command::CLines(_) => {
            // its from_ already does it
            return;
        }
        Command::CExplain(_co) => {
            append_telemetry(TelemetryEvent::ExplainNotification {
                source: "explain".to_string(),
                payload: arg.to_string(),
            });
            return;
        }
        _ => {}
    }

    // ok, below is a complicated way to say:  if we got a searcher_in, use it.  else try to extract
    // it from commands that have a `searcher` field
    let mut s_searcher = "".to_string();
    let mut r_searcher: Option<&Searcher> = None;
    if let Some(tmp) = searcher_in {
        r_searcher = Some(tmp);
    } else {
        match command {
            Command::CMore(CMore { searcher, .. })
            | Command::CLess(CLess { searcher, .. })
            | Command::CAll(CAll { searcher, .. })
            | Command::CAnd(CAnd { searcher, .. })
            | Command::CImports(CImports { searcher, .. }) => {
                r_searcher = Some(searcher);
            }
            _ => {}
        }
    }
    if let Some(searcher) = r_searcher {
        s_searcher = format!("{}", searcher);
    }

    let qualifier = command.qualifier().map(|qualifier| match qualifier {
        CommandQualifier::GrepQualifier(grep) => {
            format!("{}", grep)
        }
        CommandQualifier::DeclareQualifier(_) => unreachable!("unexpected `declare` handling"),
    });

    let unrecognized = command.qualifier().map(|qualifier| match qualifier {
        CommandQualifier::GrepQualifier(grep) => grep.unrecognized.clone(),
        CommandQualifier::DeclareQualifier(_) => unreachable!("unexpected `declare` handling"),
    });

    let unrecognized = unrecognized.unwrap_or("".to_string());

    let tmp = TelemetryEvent::GenericCommandNotification {
        name: source.to_string(),
        arg: arg.to_string(),
        qualifier: qualifier.unwrap_or("❓qualifier???".to_string()),
        unrecognized,
        searcher: s_searcher,
    };
    append_telemetry(tmp);
}

/// Parses a single command string into a typed command value.
pub fn make_command(arg: &str) -> Command {
    // build a Command from string with a separator (current com::payload or com::payload::payload..)

    let segments = split_command_fields(arg);
    let segment_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
    let command = match segment_refs.as_slice() {
        [prefix, arg0, flag] if matches!(*prefix, command_prefix::AND | command_prefix::AND1) => {
            commands::search::from_search(CommandVariant::CAnd, arg0, flag, arg)
        }
        [command_prefix::AND, arg0] | [command_prefix::AND1, arg0] => {
            commands::search::from_search(CommandVariant::CAnd, arg0, "", arg)
        }
        [command_prefix::LINES, arg0, flag] => {
            let (telemetry_event, command) = commands::lines::from_arg(arg, arg0, flag);
            append_telemetry(telemetry_event);
            return command;
        }
        [command_prefix::MACROS, arg0] => {
            let (telemetry_event, command) = commands::macros::from_arg(arg, arg0);
            append_telemetry(telemetry_event);
            return command;
        }
        [command_prefix::LINES, arg0] => {
            let (telemetry_event, command) = commands::lines::from_arg(arg, arg0, "");
            append_telemetry(telemetry_event);
            return command;
        }
        [command_prefix::MORE, arg0, flag] | [command_prefix::MORE1, arg0, flag] => {
            commands::search::from_search(CommandVariant::CMore, arg0, flag, arg)
        }
        [command_prefix::MORE, arg0] | [command_prefix::MORE1, arg0] => {
            commands::search::from_search(CommandVariant::CMore, arg0, "", arg)
        }
        [command_prefix::LESS, arg0, flag] | [command_prefix::LESS1, arg0, flag] => {
            commands::search::from_search(CommandVariant::CLess, arg0, flag, arg)
        }
        [command_prefix::LESS, arg0] | [command_prefix::LESS1, arg0] => {
            commands::search::from_search(CommandVariant::CLess, arg0, "", arg)
        }
        [command_prefix::DELETE, arg0, flag] | [command_prefix::DELETE1, arg0, flag] => {
            commands::search::from_search(CommandVariant::CDelete, arg0, flag, arg)
        }
        [command_prefix::DELETE, arg0] | [command_prefix::DELETE1, arg0] => {
            commands::search::from_search(CommandVariant::CDelete, arg0, "", arg)
        }
        [command_prefix::DELETE] | [command_prefix::DELETE1] => {
            commands::search::from_search(CommandVariant::CDelete, ".", "", arg)
        }
        [command_prefix::ALL, arg0, flag] | [command_prefix::ALL1, arg0, flag] => {
            commands::search::from_search(CommandVariant::CAll, arg0, flag, arg)
        }
        [command_prefix::ALL, arg0] | [command_prefix::ALL1, arg0] => {
            commands::search::from_search(CommandVariant::CAll, arg0, "", arg)
        }
        [command_prefix::ALL] | [command_prefix::ALL1] => {
            commands::search::from_search(CommandVariant::CAll, ".", "", arg)
        }
        [command_prefix::OUTPUT] | [command_prefix::OUTPUT1] => {
            commands::search::from_search(CommandVariant::CAll, ".", "N", arg)
        }
        [prefix, pattern, replace_with, flags]
            if matches!(*prefix, command_prefix::CHANGE | command_prefix::CHANGE1) =>
        {
            commands::change::from_change(pattern, replace_with, flags)
        }
        [command_prefix::CHANGE, pattern, replace_with]
        | [command_prefix::CHANGE1, pattern, replace_with] => {
            commands::change::from_change(pattern, replace_with, "")
        }
        [prefix, pattern, flags]
            if matches!(
                *prefix,
                command_prefix::DECLARATIONS | command_prefix::DECLARATIONS1
            ) =>
        {
            commands::declarations::from_declarations(pattern, flags, arg)
        }
        [prefix, pattern]
            if matches!(
                *prefix,
                command_prefix::DECLARATIONS | command_prefix::DECLARATIONS1
            ) =>
        {
            commands::declarations::from_declarations(pattern, "", arg)
        }
        [command_prefix::DECLARATIONS] | [command_prefix::DECLARATIONS1] => {
            commands::declarations::from_declarations("", "", arg)
        }
        [prefix, pattern, flags]
            if matches!(*prefix, command_prefix::IMPORTS | command_prefix::IMPORTS1) =>
        {
            commands::imports::from_import(pattern, flags, arg)
        }
        [prefix, pattern]
            if matches!(*prefix, command_prefix::IMPORTS | command_prefix::IMPORTS1) =>
        {
            commands::imports::from_import(pattern, "", arg)
        }
        [command_prefix::IMPORTS] | [command_prefix::IMPORTS1] => {
            commands::imports::from_import(".", "", arg)
        }
        [prefix, pattern, content, flags]
            if matches!(*prefix, command_prefix::PREPEND | command_prefix::PREPEND1) =>
        {
            commands::appendprepend::from_prepend(pattern, content, flags)
        }
        [prefix, pattern, content]
            if matches!(*prefix, command_prefix::PREPEND | command_prefix::PREPEND1) =>
        {
            commands::appendprepend::from_prepend(pattern, content, "")
        }
        [prefix, pattern, content, flags]
            if matches!(*prefix, command_prefix::APPEND | command_prefix::APPEND1) =>
        {
            commands::appendprepend::from_append(pattern, content, flags)
        }
        [prefix, pattern, content]
            if matches!(*prefix, command_prefix::APPEND | command_prefix::APPEND1) =>
        {
            commands::appendprepend::from_append(pattern, content, "")
        }
        [command_prefix::INVERT]
        | [command_prefix::INVERT1]
        | [command_prefix::INVERT, ""]
        | [command_prefix::INVERT1, ""] => Command::CInvert(CInvert),
        [command_prefix::EXPLAIN] | [command_prefix::EXPLAIN, _] => Command::CExplain(CExplain),

        _ => {
            let noop = CNoop {
                text: arg.to_string(),
                message: "unrecognized command".to_string(),
            };
            append_telemetry(TelemetryEvent::NoopNotification {
                payload: arg.to_string(),
                received: arg.to_string(),
                cause: "not understood".to_string(),
            });
            Command::CNoop(noop)
        }
    };

    append_generic_command_telemetry(&command, arg, None);
    command
}

fn split_command_fields(arg: &str) -> Vec<String> {
    // separator is `::` so m::foobar => `['m','foobar']`
    // to use `::` itself in any of the fields, use `~::` as an escape.
    // note:  the Python regex to split on `::` but not `~::` is `(?<!~)::`
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    let bytes = arg.as_bytes();

    while index < bytes.len() {
        if index + 3 <= bytes.len()
            && bytes[index] == b'~'
            && bytes[index + 1] == b':'
            && bytes[index + 2] == b':'
        {
            current.push_str(c::SEP);
            index += 3;
            continue;
        }

        if index + 2 <= bytes.len() && bytes[index] == b':' && bytes[index + 1] == b':' {
            segments.push(current);
            current = String::new();
            index += 2;
            continue;
        }
        let ch = arg[index..].chars().next().unwrap();
        current.push(ch);
        index += ch.len_utf8();
    }
    segments.push(current);
    segments
}

fn preformat(
    arg0: &str,
    qualifier: &common::GrepCommandQualifier,
    grep_shortcodes: &[GrepShortCodeDef],
) -> String {
    if qualifier.preformat {
        grep_shortcodes
            .iter()
            .fold(arg0.to_string(), |result, grep_shortcode| {
                result.replace(grep_shortcode.macrocode, grep_shortcode.expansion)
            })
    } else {
        arg0.to_string()
    }
}

// load commands from files pointed at by macro::<some path>
fn expand_macros(args: &[String]) -> Vec<String> {
    let mut res = Vec::new();
    for v in args {
        if v.starts_with("macro::") {
            res.push(v.to_string());
            match v.split("::").collect::<Vec<_>>().as_slice() {
                ["macro", path_] | ["macro", path_, _] => {
                    match utilities::parse_commands_file(path_) {
                        Ok(li_macro) => {
                            // dbg!(&li_macro);
                            for macro_ in li_macro {
                                res.push(macro_);
                            }
                        }
                        Err(e) => {
                            eprintln!("error loading {}", e);
                        }
                    };
                }
                _ => {
                    println!("unexpected format");
                }
            }
        } else {
            res.push(v.to_string());
        }
    }
    // dbg!(&res);
    res
}

/// Converts command argument strings into executable commands.
pub fn commands_factory(args: &[String]) -> Vec<Command> {
    let args = expand_macros(args);
    args.iter().map(|arg| make_command(arg)).collect()
}

#[cfg(test)]
mod tests {
    use super::preformat;
    use super::split_command_fields;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    use crate::Search;
    use crate::common::{
        CheckConditions, GrepCommandQualifier, LineStatus, MetaInfo, calc_indent,
        with_global_config,
    };
    use crate::constants as c;

    static TEST_GLOBAL_CONFIG_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    use crate::language_api::{DefaultLanguageHelper, LanguageHelper};

    #[test]
    fn language_api_has_default_parse_content_to_hashtree() {
        let helper = DefaultLanguageHelper;
        let result = helper.parse_content_to_hashtree(b"def hello():\n    return 1\n");
        assert!(result.is_empty());
    }

    #[test]
    fn built_in_helper_implements_phase1_api_boundary() {
        let helper = crate::treesitterparser::Helper::Python(crate::languages::python::HelperPython);
        let result = helper.parse_content_to_hashtree(b"def hello():\n    return 1\n");
        assert!(!result.is_empty());
    }

    fn make_line_with_key(key: &str) -> LineStatus {
        LineStatus {
            meta: MetaInfo {
                key: key.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn set_test_global_config(grep_shortcodes: bool, file_path: &str) {
        with_global_config(|global| {
            global.grep_shortcodes = grep_shortcodes;
            global.file_path = file_path.to_string();
            global.verbose = false;
            global.debug = false;
        });
    }

    fn grep_qualifier(preformat: bool) -> GrepCommandQualifier {
        GrepCommandQualifier {
            after: 0,
            before: 0,
            preformat,
            fixed_string: false,
            regex_flags: None,
            set_key: None,
            user_confirmation: false,
            showowner: false,
            unrecognized: String::new(),
            conditions_checker: CheckConditions::new(),
        }
    }

    fn assert_split(input: &str, expected: &[&str]) {
        let actual = split_command_fields(input);
        let actual_refs: Vec<&str> = actual.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_refs, expected);
    }

    #[test]
    fn split_command_fields_basic_two_parts() {
        assert_split("m::foobar", &["m", "foobar"]);
    }

    #[test]
    fn split_command_fields_three_parts() {
        assert_split("c::old::new", &["c", "old", "new"]);
    }

    #[test]
    fn split_command_fields_with_empty_middle_field() {
        assert_split("c::::new", &["c", "", "new"]);
    }

    #[test]
    fn split_command_fields_with_trailing_separator() {
        assert_split("m::", &["m", ""]);
    }

    #[test]
    fn split_command_fields_with_no_separator() {
        assert_split("declarations", &["declarations"]);
    }

    #[test]
    fn split_command_fields_empty_input() {
        assert_split("", &[""]);
    }

    #[test]
    fn split_command_fields_with_escaped_separator() {
        assert_split("m::Command~::CNoop::i", &["m", "Command::CNoop", "i"]);
    }

    #[test]
    fn preformat_expands_all_configured_grep_shortcodes() {
        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");

        let actual = preformat(
            "alpha~~s.beta..gamma~~o.delta",
            &grep_qualifier(true),
            c::grep_shortcodes::searchfield(),
        );

        assert_eq!(actual, "alpha +beta.*gamma|delta");
    }

    #[test]
    fn preformat_respects_qualifier_override_when_grep_shortcodes_disabled() {
        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let qualifier = grep_qualifier(true);

        let actual = preformat(
            "alpha~~s.beta..gamma",
            &qualifier,
            c::grep_shortcodes::searchfield(),
        );

        assert_eq!(actual, "alpha +beta.*gamma");
    }

    #[test]
    fn preformat_leaves_pattern_unchanged_when_formatting_disabled() {
        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");

        let actual = preformat(
            "alpha~~s.beta..gamma",
            &grep_qualifier(false),
            c::grep_shortcodes::searchfield(),
        );

        assert_eq!(actual, "alpha~~s.beta..gamma");
    }

    #[test]
    fn make_command_more_uses_searchfield_macros() {
        use crate::CMore;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");
        let cmd = make_command("m::alpha~~s.beta");

        let Command::CMore(CMore { searcher, .. }) = cmd else {
            panic!("expected CMore");
        };

        assert!(searcher.search("alpha beta"));
    }

    #[test]
    fn make_command_more_invalid_regex_returns_noop_with_error_message() {
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("m::[");

        let Command::CNoop(noop) = cmd else {
            panic!("expected CNoop");
        };

        assert!(
            !noop.message.is_empty(),
            "expected regex error message in CNoop"
        );
        assert!(
            noop.message.to_lowercase().contains("regex"),
            "expected regex error details, got: {}",
            noop.message
        );
    }

    #[test]
    fn make_command_all_uses_searchfield_macros() {
        use crate::CAll;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");
        let cmd = make_command("a::alpha~~s.beta");

        let Command::CAll(CAll { searcher, .. }) = cmd else {
            panic!("expected CAll");
        };

        assert!(searcher.search("alpha beta"));
    }

    #[test]
    fn make_command_and_uses_searchfield_macros() {
        use crate::CAnd;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");
        let cmd = make_command("and::alpha~~s.beta");

        let Command::CAnd(CAnd { searcher, .. }) = cmd else {
            panic!("expected CAnd");
        };

        assert!(searcher.search("alpha beta"));
    }

    #[test]
    fn make_command_delete_uses_searchfield_macros() {
        use crate::CDelete;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");
        let cmd = make_command("delete::alpha~~s.beta");

        let Command::CDelete(CDelete { searcher, .. }) = cmd else {
            panic!("expected CDelete");
        };

        assert!(searcher.search("alpha beta"));
    }

    #[test]
    fn make_command_prepend_dispatches_to_inserter() {
        use crate::Command;
        use crate::make_command;

        let cmd = make_command("prepend::foo::bar");
        assert!(matches!(cmd, Command::CInserter(_)));
    }

    #[test]
    fn make_command_append_short_dispatches_to_inserter() {
        use crate::Command;
        use crate::make_command;

        let cmd = make_command("app::foo::bar");
        assert!(matches!(cmd, Command::CInserter(_)));
    }

    #[test]
    fn make_command_import_uses_name_searchfield_macros() {
        use crate::CImports;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");
        let cmd = make_command("i::alpha~~s.beta");

        let Command::CImports(CImports { searcher, .. }) = cmd else {
            panic!("expected CImports");
        };

        assert!(!searcher.search("alpha beta"));
    }

    #[test]
    fn make_command_declare_uses_name_searchfield_macros() {
        use crate::CDeclarations;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(true, "sample.rs");
        let cmd = make_command("d::alpha~~s.beta");

        let Command::CDeclarations(CDeclarations { searcher, .. }) = cmd else {
            panic!("expected CDeclarations");
        };

        assert!(!searcher.search("alpha beta"));
    }

    #[test]
    fn calc_indent_empty_string() {
        let result = calc_indent("");
        assert_eq!(result, -1);
    }

    #[test]
    fn calc_indent_no_indentation() {
        let result = calc_indent("hello world");
        assert_eq!(result, 0);
    }

    #[test]
    fn calc_indent_single_space() {
        let result = calc_indent(" hello");
        assert_eq!(result, 1);
    }

    #[test]
    fn calc_indent_multiple_spaces() {
        let result = calc_indent("    hello");
        assert_eq!(result, 4);
    }

    #[test]
    fn calc_indent_single_tab() {
        let result = calc_indent("\thello");
        assert_eq!(result, 1);
    }

    #[test]
    fn calc_indent_multiple_tabs() {
        let result = calc_indent("\t\t\thello");
        assert_eq!(result, 3);
    }

    #[test]
    fn calc_indent_mixed_tabs_and_spaces() {
        let result = calc_indent("\t  \thello");
        assert_eq!(result, 4);
    }

    #[test]
    fn calc_indent_only_whitespace() {
        let result = calc_indent("    ");
        assert_eq!(result, -1);
    }

    #[test]
    fn calc_indent_only_tabs() {
        let result = calc_indent("\t\t");
        assert_eq!(result, -1);
    }

    #[test]
    fn calc_indent_newline_as_leading_whitespace() {
        let result = calc_indent("\n hello");
        assert_eq!(result, 2);
    }

    #[test]
    fn calc_indent_carriage_return_as_leading_whitespace() {
        let result = calc_indent("\r hello");
        assert_eq!(result, 2);
    }

    #[test]
    fn calc_indent_mixed_whitespace_types() {
        let result = calc_indent(" \t \n \r hello");
        assert_eq!(result, 7);
    }

    #[test]
    fn calc_indent_large_indentation() {
        let result = calc_indent(&"                                   code".to_string());
        assert_eq!(result, 35);
    }

    #[test]
    fn cmore_set_key_applied_to_matching_lines() {
        use crate::CommandActions;
        use crate::commands_factory;
        use crate::common::LineStatus;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let args: Vec<String> = vec!["more::hello::sk=hit.".to_string()];
        let commands = commands_factory(&args);

        let lines = vec![
            LineStatus {
                line_number: 1,
                visible: false,
                line: "hello world".to_string(),
                ..Default::default()
            },
            LineStatus {
                line_number: 2,
                visible: false,
                line: "goodbye world".to_string(),
                ..Default::default()
            },
        ];

        let result = commands[0].apply(lines, &std::collections::HashMap::new());

        assert_eq!(result[0].meta.key, "hit"); // matched line gets key
        assert_eq!(result[1].meta.key, ""); // non-matching line unchanged
    }

    #[test]
    fn make_command_more_parses_setkey_from_flag_field() {
        use crate::CMore;
        use crate::Command;
        use crate::common::CommandQualifier;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("m::init::sk=ini.");

        let Command::CMore(CMore {
            qualifier: CommandQualifier::GrepQualifier(grep),
            ..
        }) = cmd
        else {
            panic!("expected CMore with GrepQualifier");
        };

        assert_eq!(grep.set_key, Some("ini".to_string()));
    }

    #[test]
    fn calc_indent_with_leading_zeros_like_string() {
        let result = calc_indent(&"    000123".to_string());
        assert_eq!(result, 4);
    }

    #[test]
    fn make_command_less_parses_wherekey_from_flag_field() {
        use crate::CLess;
        use crate::Command;
        use crate::common::CommandQualifier;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("less::init::wk=priv.");

        let Command::CLess(CLess {
            qualifier: CommandQualifier::GrepQualifier(grep),
            ..
        }) = cmd
        else {
            panic!("expected CLess with GrepQualifier");
        };

        let matching = make_line_with_key("priv");
        let non_matching = make_line_with_key("other");
        assert!(grep.conditions_checker.check(&matching));
        assert!(!grep.conditions_checker.check(&non_matching));
    }

    #[test]
    fn make_command_invert_is_one_field_command() {
        use crate::CInvert;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("invert");

        let Command::CInvert(CInvert) = cmd else {
            panic!("expected CInvert");
        };
    }

    #[test]
    fn make_command_invert_trailing_separator_is_allowed() {
        use crate::CInvert;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("invert::");

        let Command::CInvert(CInvert) = cmd else {
            panic!("expected CInvert");
        };
    }

    #[test]
    fn make_command_explain_is_one_field_command() {
        use crate::CExplain;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("explain");

        let Command::CExplain(CExplain) = cmd else {
            panic!("expected CExplain");
        };
    }

    #[test]
    fn make_command_explain_trailing_field_is_allowed() {
        use crate::CExplain;
        use crate::Command;
        use crate::make_command;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let cmd = make_command("explain::s");

        let Command::CExplain(CExplain) = cmd else {
            panic!("expected CExplain");
        };
    }

    #[test]
    fn less_wherekey_only_hides_matching_keyed_lines() {
        use crate::CommandActions;
        use crate::commands_factory;
        use crate::common::LineStatus;

        let _guard = TEST_GLOBAL_CONFIG_LOCK.lock().expect("test lock poisoned");
        set_test_global_config(false, "sample.rs");
        let args: Vec<String> = vec![
            "more::def __::sk=priv.".to_string(),
            "less::i::wk=priv.".to_string(),
        ];
        let commands = commands_factory(&args);

        let mut lines = vec![
            LineStatus {
                line_number: 1,
                visible: true,
                line: "def __init__(self):".to_string(),
                ..Default::default()
            },
            LineStatus {
                line_number: 2,
                visible: true,
                line: "if something:".to_string(),
                ..Default::default()
            },
            LineStatus {
                line_number: 3,
                visible: true,
                line: "print('x')".to_string(),
                ..Default::default()
            },
        ];

        for command in &commands {
            lines = command.apply(lines, &std::collections::HashMap::new());
        }

        assert!(!lines[0].visible);
        assert_eq!(lines[0].meta.key, "priv");

        assert!(lines[1].visible);
        assert_eq!(lines[1].meta.key, "");

        assert!(lines[2].visible);
        assert_eq!(lines[2].meta.key, "");
    }
}
