use crate::commands::prelude::*;

#[derive(Debug)]
/// Hides lines that match the configured pattern.
pub struct CLess {
    pub(crate) searcher: Searcher,
    /// instructions for the command
    pub qualifier: CommandQualifier,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::LESS,
        mutates_line_text: false,
        flags: commandflags::searchflags(),
    }
}

impl CommandActions for CLess {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        fn f_calculate_visibility(visible: bool, hit: bool) -> bool {
            visible && !hit
        }

        let qualifier = match &self.qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CLess requires GrepQualifier"),
        };

        let (_, result_lines) = search_vector(
            f_calculate_visibility,
            &self.searcher,
            lines,
            CommandVariant::CLess,
            qualifier,
        );

        result_lines
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}
