use crate::Search;
use crate::commands::prelude::*;
use inquire::Confirm;

#[derive(Debug)]
/// Deletes visible lines matching the configured pattern.
pub struct CDelete {
    pub(crate) searcher: Searcher,
    /// instructions for the command
    pub qualifier: CommandQualifier,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::DELETE,
        mutates_line_text: true,
        flags: commandflags::searchflags(),
    }
}

impl CDelete {
    // ask user to confirm deletion, line by line
    fn user_confirmation(&self, line: &LineStatus, _display_lines: &[LineStatus]) -> bool {
        // !!!TODO!!! display before/after context
        let message = format!("delete {}", line.line,);
        let ans = Confirm::new(&message).with_default(true).prompt();
        ans.unwrap_or_default()
    }
}

impl CommandActions for CDelete {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        let qualifier = match &self.qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CDelete requires GrepQualifier"),
        };

        let display_lines = if qualifier.user_confirmation {
            Some(lines.clone())
        } else {
            None
        };

        // Filter out visible lines that match the pattern
        lines
            .into_iter()
            .filter(|line_status| {
                let hit = self.searcher.search(&line_status.line)
                    && qualifier.conditions_checker.check(line_status);

                if qualifier.user_confirmation {
                    let display_lines = display_lines
                        .as_ref()
                        .expect("display_lines must exist when user_confirmation is enabled");
                    !(line_status.visible && hit)
                        || !self.user_confirmation(line_status, display_lines)
                } else {
                    !line_status.visible || !hit
                }
                // Keep the line if it's not visible, or if it's visible and doesn't match
            })
            .collect()
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}
