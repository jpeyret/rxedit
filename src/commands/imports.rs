use std::collections::HashSet;

use crate::Search;
use crate::commands::prelude::*;
use crate::common::{set_visibles, update_meta_key};
use crate::constants as c;
use crate::utilities;
use crate::{CNoop, Command, new_searcher};

#[derive(Debug)]
/// Makes import lines visible when any stored import segment matches the search pattern.
pub struct CImports {
    pub(crate) searcher: Searcher,
    /// instructions for the command
    pub qualifier: CommandQualifier,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::IMPORTS,
        mutates_line_text: false,
        flags: commandflags::searchflags(),
    }
}

pub(crate) fn from_import(pattern: &str, flags: &str, arg: &str) -> Command {
    let (keep_pattern, skip_pattern) = utilities::split_name_negation(pattern);

    let seed = new_searcher(
        keep_pattern,
        skip_pattern,
        flags,
        c::commandflags::searchflags(),
        c::grep_shortcodes::name_searchfield(),
    );

    if let Ok(seed) = seed {
        let qualifier =
            CommandQualifier::GrepQualifier(seed.qualifier.expect("missing import qualifier"));

        Command::CImports(CImports {
            searcher: seed.searcher,
            qualifier,
        })
    } else {
        use crate::common::{TelemetryEvent, append_telemetry};

        let error = seed.expect_err("error must exist for failed searcher build");
        let error_message = error.to_string();

        append_telemetry(TelemetryEvent::NoopNotification {
            payload: pattern.to_string(),
            received: arg.to_string(),
            cause: error_message.clone(),
        });

        Command::CNoop(CNoop {
            text: arg.to_string(),
            message: error_message,
        })
    }
}

impl CommandActions for CImports {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        let qualifier = match &self.qualifier {
            CommandQualifier::GrepQualifier(grep) => grep,
            _ => panic!("CImport requires GrepQualifier"),
        };

        let hit_hashtree_keys: Vec<usize> = hashtree
            .iter()
            .filter_map(|(key, parsed)| {
                let mut skip = false;
                let parsed = parsed.as_imports()?;

                let key_matches = lines
                    .get(*key)
                    .map(|line_status| qualifier.conditions_checker.check(line_status))
                    .unwrap_or_else(|| qualifier.conditions_checker.conds.is_empty());

                let import_match = parsed.names.iter().any(|name| self.searcher.search(name));

                if import_match {
                    // if we do have an overall match, let's make sure nothing calls for a skip
                    skip = parsed.names.iter().any(|name| self.searcher.skip(name));
                }

                if key_matches && import_match && !skip {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect();

        let saved: HashSet<usize> = hit_hashtree_keys.iter().copied().collect();
        let mut result_lines = set_visibles(lines, &saved);
        result_lines = post_search(qualifier, &hit_hashtree_keys, result_lines);

        if let Some(set_key) = &qualifier.set_key {
            result_lines = update_meta_key(result_lines, &saved, set_key);
        }

        if qualifier.showowner && has_valid_treesitter(hashtree) {
            result_lines = show_owner_from_hits(&hit_hashtree_keys, result_lines, hashtree);
        }

        result_lines
    }
    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}
