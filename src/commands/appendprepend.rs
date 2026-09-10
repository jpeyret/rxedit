use crate::commands::prelude::*;
use crate::common::{
    CommandQualifier, GrepCommandQualifier, TelemetryEvent, append_telemetry, calc_indent,
};
use crate::constants as c;
use crate::{Command, Searcher, new_searcher};
use inquire::Confirm;
use regex::Regex;

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: "append",
        mutates_line_text: true,
        flags: commandflags::insert_flags(),
    }
}

#[derive(Debug)]
/// Insert strategy for prepend/append commands.
pub enum AppendPrepend {
    /// Regex-based insertion.
    Regex {
        /// Whether to insert before the matched line.
        insert_before: bool,
        /// Whether to preserve indentation from the matched line.
        indent_keep: bool,
        /// Regex pattern used to match lines.
        patre: Regex,
        /// Text to insert.
        content: String,
    },
    /// Literal substring-based insertion.
    Literal {
        /// Whether to insert before the matched line.
        insert_before: bool,
        /// Whether to preserve indentation from the matched line.
        indent_keep: bool,
        /// Literal pattern used to match lines.
        pattern: String,
        /// Text to insert.
        content: String,
    },
}

impl AppendPrepend {
    fn is_match(&self, line: &str) -> bool {
        match self {
            AppendPrepend::Regex { patre, .. } => patre.is_match(line),
            AppendPrepend::Literal { pattern, .. } => line.contains(pattern),
        }
    }

    fn insert_before(&self) -> bool {
        match self {
            AppendPrepend::Regex { insert_before, .. }
            | AppendPrepend::Literal { insert_before, .. } => *insert_before,
        }
    }

    fn indent_keep(&self) -> bool {
        match self {
            AppendPrepend::Regex { indent_keep, .. }
            | AppendPrepend::Literal { indent_keep, .. } => *indent_keep,
        }
    }

    fn content(&self) -> &str {
        match self {
            AppendPrepend::Regex { content, .. } | AppendPrepend::Literal { content, .. } => {
                content
            }
        }
    }
}

#[derive(Debug)]
/// Inserts text before or after matching visible lines.
pub struct CInserter {
    pub(crate) inserter: AppendPrepend,
    /// instructions for the command
    pub qualifier: Option<CommandQualifier>,
}

impl CInserter {
    /// Returns the source command prefix for telemetry.
    pub fn source_prefix(&self) -> &'static str {
        if self.inserter.insert_before() {
            command_prefix::PREPEND
        } else {
            command_prefix::APPEND
        }
    }

    fn user_confirmation(&self, line: &LineStatus, inserted: &str) -> bool {
        let action = if self.inserter.insert_before() {
            "prepend"
        } else {
            "append"
        };
        let message = format!("{} `{}` around `{}`", action, inserted, line.line);
        Confirm::new(&message)
            .with_default(true)
            .prompt()
            .unwrap_or_default()
    }

    fn grep_qualifier(&self) -> Option<&GrepCommandQualifier> {
        self.qualifier.as_ref().map(|qualifier| match qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CInserter requires GrepQualifier when qualifier is present"),
        })
    }
}

fn apply_indent(content: &str, indent_spaces: i32) -> String {
    if indent_spaces <= 0 {
        content.to_string()
    } else {
        format!("{}{}", " ".repeat(indent_spaces as usize), content)
    }
}

fn inserted_line_text(base_content: &str, matched_line: &str, indent_keep: bool) -> String {
    if !indent_keep {
        return base_content.to_string();
    }
    let indent = calc_indent(matched_line);
    apply_indent(base_content, indent.max(0))
}

fn from_insert(pattern: &str, content: &str, flags: &str, insert_before: bool) -> Command {
    let command_name = if insert_before {
        command_prefix::PREPEND
    } else {
        command_prefix::APPEND
    };

    let result = new_searcher(
        pattern,
        None,
        flags,
        c::commandflags::insert_flags(),
        c::grep_shortcodes::searchfield(),
    );

    if let Ok(seed) = result {
        let qualifier = seed.qualifier.expect("missing insert qualifier");
        let indent_keep = flags.contains(c::commandflags::INDENT_KEEP.value);

        let inserter = match seed.searcher {
            Searcher::RegexSearcher(re) => AppendPrepend::Regex {
                insert_before,
                indent_keep,
                patre: re.patre,
                content: content.to_string(),
            },
            Searcher::ContainsSearcher(contains) => AppendPrepend::Literal {
                insert_before,
                indent_keep,
                pattern: contains.pattern,
                content: content.to_string(),
            },
            _ => unreachable!("insert only supports single keep pattern"),
        };

        Command::CInserter(CInserter {
            inserter,
            qualifier: Some(CommandQualifier::GrepQualifier(qualifier)),
        })
    } else {
        let error_message = result
            .expect_err("error must exist for failed inserter build")
            .to_string();

        append_telemetry(TelemetryEvent::NoopNotification {
            payload: pattern.to_string(),
            received: format!("{}::{}::{}::{}", command_name, pattern, content, flags),
            cause: error_message.clone(),
        });

        Command::CNoop(crate::CNoop {
            text: format!("{}::{}::{}::{}", command_name, pattern, content, flags),
            message: error_message,
        })
    }
}

pub(crate) fn from_prepend(pattern: &str, content: &str, flags: &str) -> Command {
    from_insert(pattern, content, flags, true)
}

pub(crate) fn from_append(pattern: &str, content: &str, flags: &str) -> Command {
    from_insert(pattern, content, flags, false)
}

impl CommandActions for CInserter {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        let grep_qualifier = self.grep_qualifier();
        let user_confirmation = grep_qualifier
            .map(|grep| grep.user_confirmation)
            .unwrap_or(false);

        let mut result = Vec::with_capacity(lines.len());

        for line_status in lines {
            let key_matches = grep_qualifier
                .map(|grep| grep.conditions_checker.check(&line_status))
                .unwrap_or(true);

            let hit =
                line_status.visible && key_matches && self.inserter.is_match(&line_status.line);

            if hit {
                let inserted = inserted_line_text(
                    self.inserter.content(),
                    &line_status.line,
                    self.inserter.indent_keep(),
                );

                let should_insert = if user_confirmation {
                    self.user_confirmation(&line_status, &inserted)
                } else {
                    true
                };

                if should_insert && self.inserter.insert_before() {
                    result.push(LineStatus {
                        line_number: line_status.line_number,
                        visible: true,
                        line: inserted.clone(),
                        meta: Default::default(),
                    });
                }

                result.push(line_status.clone());

                if should_insert && !self.inserter.insert_before() {
                    result.push(LineStatus {
                        line_number: line_status.line_number,
                        visible: true,
                        line: inserted,
                        meta: Default::default(),
                    });
                }
            } else {
                result.push(line_status);
            }
        }

        result
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::MetaInfo;

    fn line(num: usize, visible: bool, text: &str) -> LineStatus {
        LineStatus {
            line_number: num,
            visible,
            line: text.to_string(),
            meta: MetaInfo::default(),
        }
    }

    fn apply_cmd(cmd: &CInserter, lines: Vec<LineStatus>) -> Vec<LineStatus> {
        cmd.apply(lines, &HashMap::new())
    }

    #[test]
    fn prepend_inserts_before_matching_visible_line() {
        let cmd = CInserter {
            inserter: AppendPrepend::Literal {
                insert_before: true,
                indent_keep: false,
                pattern: "foo".to_string(),
                content: "new".to_string(),
            },
            qualifier: None,
        };

        let lines = vec![line(1, true, "foo")];
        let out = apply_cmd(&cmd, lines);

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].line, "new");
        assert_eq!(out[1].line, "foo");
    }

    #[test]
    fn append_inserts_after_matching_visible_line() {
        let cmd = CInserter {
            inserter: AppendPrepend::Literal {
                insert_before: false,
                indent_keep: false,
                pattern: "foo".to_string(),
                content: "new".to_string(),
            },
            qualifier: None,
        };

        let lines = vec![line(1, true, "foo")];
        let out = apply_cmd(&cmd, lines);

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].line, "foo");
        assert_eq!(out[1].line, "new");
    }

    #[test]
    fn indent_keep_applies_matching_line_indent() {
        let cmd = CInserter {
            inserter: AppendPrepend::Literal {
                insert_before: true,
                indent_keep: true,
                pattern: "BAR".to_string(),
                content: "# note".to_string(),
            },
            qualifier: None,
        };

        let lines = vec![line(2, true, "  BAR")];
        let out = apply_cmd(&cmd, lines);

        assert_eq!(out[0].line, "  # note");
    }

    #[test]
    fn does_not_insert_for_invisible_lines() {
        let cmd = CInserter {
            inserter: AppendPrepend::Literal {
                insert_before: true,
                indent_keep: false,
                pattern: "foo".to_string(),
                content: "new".to_string(),
            },
            qualifier: None,
        };

        let lines = vec![line(1, false, "foo")];
        let out = apply_cmd(&cmd, lines);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, "foo");
    }
}
