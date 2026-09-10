use std::collections::HashSet;

use crate::commands::prelude::*;

#[derive(Debug)]
/// Shows lines that are already visible and also match the pattern.
pub struct CAnd {
    pub(crate) searcher: Searcher,
    /// instructions for the command
    pub qualifier: CommandQualifier,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::AND,
        mutates_line_text: false,
        flags: commandflags::and_flags(),
    }
}

impl CommandActions for CAnd {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        fn f_calculate_visibility(visible: bool, hit: bool) -> bool {
            visible && hit
        }

        let qualifier = match &self.qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CAnd requires GrepQualifier"),
        };

        let visibles: HashSet<usize> = lines
            .iter()
            .enumerate()
            .map(|(idx, line_status)| {
                if line_status.visible {
                    idx
                } else {
                    usize::MAX // this wont be found, so the set intersection will drop it
                }
            })
            .collect();

        let (search_hits, result_lines) = search_vector(
            f_calculate_visibility,
            &self.searcher,
            lines,
            CommandVariant::CAnd,
            qualifier,
        );

        //howto- compute intersection of what was already visible vs search hits
        let hits: Vec<usize> = search_hits
            .iter()
            .filter(|n| visibles.contains(n))
            .copied()
            .collect();

        post_search(qualifier, &hits, result_lines)
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}
