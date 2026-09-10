use crate::ChangeReplacer;
use crate::commands::prelude::*;
use crate::{Command, new_replacer};
use inquire::Confirm;

#[derive(Debug)]
/// Replaces text in visible lines while preserving visibility.
pub struct CChange {
    pub(crate) replacer: ChangeReplacer,
    /// instructions for the command
    pub qualifier: Option<CommandQualifier>,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::CHANGE,
        mutates_line_text: true,
        flags: commandflags::change_flags(),
    }
}

impl CChange {
    fn user_confirmation(
        &self,
        line: &LineStatus,
        changed_line: &str,
        _display_lines: &[LineStatus],
    ) -> bool {
        let message = format!("change : {}\n  to     : {}\n", line.line, changed_line,);

        let ans = Confirm::new(&message).with_default(true).prompt();

        ans.unwrap_or_default()
    }
}

pub(crate) fn from_change(pattern: &str, replace_with: &str, flags: &str) -> Command {
    let seed = new_replacer(pattern, replace_with, flags);
    Command::CChange(CChange {
        replacer: seed.replacer,
        qualifier: Some(CommandQualifier::GrepQualifier(
            seed.qualifier.expect("missing change qualifier"),
        )),
    })
}

impl CommandActions for CChange {
    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }

    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        let grep_qualifier = self.qualifier.as_ref().map(|qualifier| match qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CChange requires GrepQualifier when qualifier is present"),
        });

        let display_lines = if grep_qualifier.map(|q| q.user_confirmation).unwrap_or(false) {
            Some(lines.clone())
        } else {
            None
        };

        let mut result = Vec::new();

        for line_status in lines {
            let key_matches = grep_qualifier
                .map(|grep| grep.conditions_checker.check(&line_status))
                .unwrap_or(true);

            if line_status.visible && key_matches {
                let changed_line = self.replacer.apply(&line_status.line);

                if changed_line != line_status.line {
                    let should_apply_change = grep_qualifier
                        .map(|grep| {
                            if grep.user_confirmation {
                                let display_lines = display_lines.as_ref().expect(
                                    "display_lines must exist when user_confirmation is enabled",
                                );
                                self.user_confirmation(&line_status, &changed_line, display_lines)
                            } else {
                                true
                            }
                        })
                        .unwrap_or(true);

                    if should_apply_change {
                        let has_linefeed = matches!(
                            &self.replacer,
                            ChangeReplacer::Regex { linefeed: true, .. }
                                | ChangeReplacer::Literal { linefeed: true, .. }
                        );

                        let changed_lines = if has_linefeed {
                            changed_line
                                .split('\n')
                                .map(|part| part.to_string())
                                .collect()
                        } else {
                            vec![changed_line]
                        };

                        for line in changed_lines {
                            let mut changed_status = line_status.clone();
                            changed_status.line = line;
                            result.push(changed_status);
                        }
                        continue;
                    }
                }
            }

            result.push(line_status);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::MetaInfo;
    use std::collections::HashMap;

    fn line_status(text: &str) -> LineStatus {
        LineStatus {
            line_number: 7,
            visible: true,
            line: text.to_string(),
            meta: MetaInfo::default(),
        }
    }

    #[test]
    fn change_keeps_single_line_replacement() {
        let cmd = CChange {
            replacer: ChangeReplacer::Literal {
                pattern: "foo".to_string(),
                replace_with: "bar".to_string(),
                global: false,
                linefeed: false,
            },
            qualifier: None,
        };

        let out = cmd.apply(vec![line_status("foo")], &HashMap::new());

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, "bar");
        assert_eq!(out[0].line_number, 7);
    }

    #[test]
    fn change_splits_replacement_on_unix_newlines() {
        let cmd = CChange {
            replacer: ChangeReplacer::Literal {
                pattern: "foo".to_string(),
                replace_with: "bar\nbaz".to_string(),
                global: false,
                linefeed: true,
            },
            qualifier: None,
        };

        let out = cmd.apply(vec![line_status("foo")], &HashMap::new());

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].line, "bar");
        assert_eq!(out[1].line, "baz");
        assert_eq!(out[0].line_number, 7);
        assert_eq!(out[1].line_number, 7);
    }

    #[test]
    fn change_raw_string_keeps_literal_backslash_n() {
        let cmd = CChange {
            replacer: ChangeReplacer::Literal {
                pattern: "foo".to_string(),
                replace_with: r"bar\nbaz".to_string(),
                global: false,
                linefeed: false,
            },
            qualifier: None,
        };

        let out = cmd.apply(vec![line_status("foo")], &HashMap::new());

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, r"bar\nbaz");
        assert_eq!(out[0].line_number, 7);
    }
}
