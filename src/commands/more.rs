use crate::commands::prelude::*;

#[derive(Debug)]
/// Makes matching lines visible while preserving existing visibility.
pub struct CMore {
    pub(crate) searcher: Searcher,
    /// instructions for the command
    /// instructions for command
    pub qualifier: CommandQualifier,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::MORE,
        mutates_line_text: false,
        flags: commandflags::searchflags(),
    }
}

impl CommandActions for CMore {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        fn f_calculate_visibility(visible: bool, hit: bool) -> bool {
            visible || hit
        }

        let qualifier = match &self.qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CMore requires GrepQualifier"),
        };

        let (hits, result_lines) = search_vector(
            f_calculate_visibility,
            &self.searcher,
            lines,
            CommandVariant::CMore,
            qualifier,
        );

        let mut result_lines = post_search(qualifier, &hits, result_lines);

        if qualifier.showowner && has_valid_treesitter(hashtree) {
            result_lines = show_owner_from_hits(&hits, result_lines, hashtree);
        }

        result_lines
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        // CommandDefinition {
        //     name: command_prefix::MORE,
        //     mutates_line_text: false,
        //     flags: commandflags::searchflags(),
        // }
        get_constant_definition()
    }
}
