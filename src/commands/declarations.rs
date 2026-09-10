//! The generic, tree-sitter aware mechanism that allows the parsing of
//! class and function declarations and also determining to which declaration
//! a given source line belongs.
//! This is driven by a source file's extension.
use std::collections::HashSet;

use crate::base::DeclarationsParentMatcher;
use crate::commands::prelude::*;
use crate::common::DeclarationsCommandQualifier;
use crate::common::{
    NoGrammarNotification, TelemetryEvent, append_telemetry, get_global_config, set_visibles,
    update_meta_key,
};
use crate::constants::{self as c, commandflags, declarations};
use crate::flag_utilities;
use crate::user_messages;
use crate::utilities;
use crate::utilities::vec_lines;
use crate::{CNoop, Command, new_searcher};
use log::error;
use log::warn;

#[derive(Debug)]
/// Makes declaration lines visible (optionally filtered by `declaration::<pattern>::` being applied to function, class or struct names).
pub struct CDeclarations {
    pub(crate) searcher: Searcher,
    /// instructions for the command
    pub qualifier: DeclarationsCommandQualifier,
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::DECLARATIONS,
        mutates_line_text: false,
        flags: commandflags::declaration_flags(),
    }
}

fn declaration_pattern_for_file(file_path: &str) -> Option<(String, String)> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str());

    match extension {
        Some("rs") => Some((declarations::RUST.to_string(), "rs".to_string())),
        Some("py") => Some((declarations::PYTHON.to_string(), "py".to_string())),
        Some(ext) if crate::treesitterparser::supports_extension_for_parsing(ext) => {
            Some((String::new(), ext.to_string()))
        }
        _ => None,
    }
}

pub(crate) fn from_declarations(pattern: &str, flags: &str, arg: &str) -> Command {
    let parents: Option<Vec<String>>;
    let declaration_pattern: String;
    let global_config = get_global_config();

    let (declaration_pattern, extension) =
        match declaration_pattern_for_file(&global_config.file_path) {
            Some((_base, extension)) => {
                (declaration_pattern, parents) = utilities::split_search_parents(pattern);
                if *c::DEBUGGING {
                    dbg!(&declaration_pattern, &parents);
                }
                (declaration_pattern, extension.to_string())
            }
            None => {
                let extension = std::path::Path::new(&global_config.file_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_string());
                let extension_text = extension.clone().unwrap_or_default();
                append_telemetry(TelemetryEvent::NoGrammarNotification(
                    NoGrammarNotification {
                        extension: extension_text.clone(),
                        source: "declarations".to_string(),
                        payload: arg.to_string(),
                        flags: flags.to_string(),
                    },
                ));
                return Command::CNoop(CNoop {
                    text: format!("pattern={pattern},flags={flags}"),
                    message: format!(
                        "declarations:  no grammar found for extension={extension_text}"
                    ),
                });
            }
        };

    let (declaration_pattern, skip_pattern) = utilities::split_name_negation(&declaration_pattern);
    let result = new_searcher(
        declaration_pattern,
        skip_pattern,
        flags,
        c::commandflags::searchflags(),
        c::grep_shortcodes::name_searchfield(),
    );

    if let Ok(seed) = result {
        let grep = seed.qualifier.expect("missing qualifier");
        let case_insensitive = grep
            .regex_flags
            .as_ref()
            .is_some_and(|flags| flags.contains('i'));
        let parent_matchers =
            match build_parent_matchers(&parents, grep.fixed_string, case_insensitive) {
                Ok(parent_matchers) => parent_matchers,
                Err(error) => {
                    let error_message = error.to_string();

                    append_telemetry(TelemetryEvent::NoopNotification {
                        payload: pattern.to_string(),
                        received: arg.to_string(),
                        cause: error_message.clone(),
                    });

                    return Command::CNoop(CNoop {
                        text: arg.to_string(),
                        message: error_message,
                    });
                }
            };
        let (unrecognized, showbody) = flag_utilities::return_contains_and_consume(
            commandflags::SHOW_BODY.value,
            &seed.unused,
        );

        let (unrecognized, where_type) = flag_utilities::return_capture_and_consume(
            &flag_utilities::WHERETYPE_REGEX,
            &unrecognized,
        );

        let qualifier: DeclarationsCommandQualifier = DeclarationsCommandQualifier {
            searchqualifier: grep,
            parents,
            parent_matchers,
            extension,
            showdoc: false,
            showbody,
            unrecognized,
            where_type,
        };

        append_telemetry(TelemetryEvent::DeclarationsNotification {
            arg: arg.to_string(),
            pattern: pattern.to_string(),
            flags: flags.to_string(),
            extension: qualifier.extension.clone(),
            qualifier: format!("{}", qualifier),
            unrecognized: qualifier.unrecognized.clone(),
            // where_type: qualifier.where_type.clone(),
            searcher: format!("{}", seed.searcher),
        });

        Command::CDeclarations(CDeclarations {
            searcher: seed.searcher,
            qualifier,
        })
    } else {
        let error = result.expect_err("error must exist for failed searcher build");
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

fn show_body_from_hashtree(
    hits: &[usize],
    lines: Vec<LineStatus>,
    hashtree: &HashMap<usize, common::Parsed>,
) -> Vec<LineStatus> {
    let mut context_indices = HashSet::new();
    let total_lines = lines.len();

    for hit in hits {
        if let Some(parsed) = hashtree
            .get(hit)
            .and_then(|parsed| parsed.as_declarations())
            && let Some((start, end)) = parsed.lines_body
        {
            if total_lines == 0 {
                continue;
            }
            let start_idx = std::cmp::min(start, total_lines - 1);
            let end_idx = std::cmp::min(end, total_lines - 1);
            for idx in start_idx..=end_idx {
                context_indices.insert(idx);
            }
        }
    }
    set_visibles(lines, &context_indices)
}

fn show_owner_from_hashtree(
    hits: &[usize],
    lines: Vec<LineStatus>,
    hashtree: &HashMap<usize, common::Parsed>,
) -> Vec<LineStatus> {
    let mut context_indices = HashSet::new();

    for hit in hits {
        let Some(parsed) = hashtree
            .get(hit)
            .and_then(|parsed| parsed.as_declarations())
        else {
            continue;
        };

        for parent_key in &parsed.parents {
            if let Some(parent_decl) = hashtree
                .get(parent_key)
                .and_then(|parsed| parsed.as_declarations())
            {
                let (sig_start, sig_end) = parent_decl.lines_signature;
                for idx in sig_start..=sig_end {
                    context_indices.insert(idx);
                }
            } else {
                context_indices.insert(*parent_key);
            }
        }
    }
    set_visibles(lines, &context_indices)
}

fn split_parent_filters(value: &str) -> Vec<&str> {
    value
        .split([',', ' '])
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect()
}

fn build_parent_matcher(
    wanted: &str,
    fixed_string: bool,
    case_insensitive: bool,
) -> Result<DeclarationsParentMatcher, regex::Error> {
    let wanted_cmp = if case_insensitive {
        wanted.to_lowercase()
    } else {
        wanted.to_string()
    };

    if fixed_string {
        let (positive, negative) = utilities::split_name_negation(&wanted_cmp);
        let positive = split_parent_filters(positive)
            .into_iter()
            .map(str::to_string)
            .collect();
        let negative = negative
            .map(split_parent_filters)
            .unwrap_or_default()
            .into_iter()
            .map(str::to_string)
            .collect();

        return Ok(DeclarationsParentMatcher::Fixed {
            positive,
            negative,
            case_insensitive,
        });
    }

    let (positive, negative) = utilities::split_name_negation(&wanted_cmp);
    let positive = regex::Regex::new(&positive.replace(',', "|"))?;
    let negative = match negative {
        Some(value) => Some(regex::Regex::new(&value.replace(',', "|"))?),
        None => None,
    };

    Ok(DeclarationsParentMatcher::Regex {
        positive,
        negative,
        case_insensitive,
    })
}

fn build_parent_matchers(
    parents: &Option<Vec<String>>,
    fixed_string: bool,
    case_insensitive: bool,
) -> Result<Option<Vec<DeclarationsParentMatcher>>, regex::Error> {
    parents
        .as_ref()
        .map(|parents| {
            parents
                .iter()
                .map(|wanted| build_parent_matcher(wanted, fixed_string, case_insensitive))
                .collect()
        })
        .transpose()
}

fn parent_matcher_allows_missing_parent(matcher: &DeclarationsParentMatcher) -> bool {
    match matcher {
        DeclarationsParentMatcher::Fixed {
            positive, negative, ..
        } => positive.is_empty() && !negative.is_empty(),
        DeclarationsParentMatcher::Regex {
            positive, negative, ..
        } => positive.as_str().is_empty() && negative.is_some(),
    }
}

fn parent_matcher_matches(matcher: &DeclarationsParentMatcher, name: &str) -> bool {
    match matcher {
        DeclarationsParentMatcher::Fixed {
            positive,
            negative,
            case_insensitive,
        } => {
            let name_cmp = if *case_insensitive {
                name.to_lowercase()
            } else {
                name.to_string()
            };

            let matches_positive = if positive.is_empty() {
                true
            } else {
                positive.iter().any(|filter| name_cmp.contains(filter))
            };

            let matches_negative = negative.iter().any(|filter| name_cmp.contains(filter));

            matches_positive && !matches_negative
        }
        DeclarationsParentMatcher::Regex {
            positive,
            negative,
            case_insensitive,
        } => {
            let name_cmp = if *case_insensitive {
                name.to_lowercase()
            } else {
                name.to_string()
            };

            let matches_positive = positive.is_match(&name_cmp);
            let matches_negative = negative
                .as_ref()
                .is_some_and(|negative_regex| negative_regex.is_match(&name_cmp));

            matches_positive && !matches_negative
        }
    }
}

fn declaration_flag_for_type(type_: &str) -> Option<char> {
    let mapped_flag = c::grammars::python::PHF_WANTED_DECLARES
        .get(type_)
        .copied()
        .or_else(|| c::grammars::rust::PHF_WANTED_DECLARES.get(type_).copied());

    mapped_flag.and_then(|value| value.chars().next())
}

fn is_supported_declaration_flag(flag: char) -> bool {
    c::grammars::python::PHF_WANTED_DECLARES
        .values()
        .any(|mapped| mapped.starts_with(flag))
        || c::grammars::rust::PHF_WANTED_DECLARES
            .values()
            .any(|mapped| mapped.starts_with(flag))
}

fn filter_by_nodetype(
    types_: &str,
    hashtree: &HashMap<usize, common::Parsed>,
    hits_declares: Vec<usize>,
) -> Vec<usize> {
    let requested_flags: HashSet<char> = types_.chars().collect();

    let has_supported_flag = requested_flags
        .iter()
        .any(|flag| is_supported_declaration_flag(*flag));

    if !has_supported_flag {
        return hits_declares;
    }

    hits_declares
        .into_iter()
        .filter(|hashtree_key| {
            hashtree
                .get(hashtree_key)
                .and_then(|parsed| parsed.as_declarations())
                .and_then(|parsed| declaration_flag_for_type(&parsed.type_))
                .is_some_and(|flag| requested_flags.contains(&flag))
        })
        .collect()
}

/// filter declarations depending on parent criteria:
/// ex:  `d::Url/__init__` means find any signatures with `__init__` whose parents match `Url`
/// ex:  `d::Url/` means find any signatures whose parents match `Url`
fn filter_parent_where(
    hashtree: &HashMap<usize, common::Parsed>,
    wanted: &[DeclarationsParentMatcher],
    hits_declares: &Vec<usize>,
) -> (Vec<usize>, HashSet<usize>) {
    let mut keeps: Vec<usize> = Vec::new();
    let mut parent_indices: HashSet<usize> = HashSet::new();

    for hashtree_key in hits_declares {
        if let Some(parsed) = hashtree
            .get(hashtree_key)
            .and_then(|parsed| parsed.as_declarations())
        {
            //if wanted is a negative only and there are no parents, keep it
            if wanted
                .first()
                .is_some_and(parent_matcher_allows_missing_parent)
                && parsed.parents.is_empty()
            {
                keeps.push(*hashtree_key);
                continue;
            }
            for (wanted, parent_idx) in wanted.iter().zip(parsed.parents.iter()) {
                if let Some(parsed_parent) = hashtree
                    .get(parent_idx)
                    .and_then(|parsed| parsed.as_declarations())
                {
                    let keep = parent_matcher_matches(wanted, &parsed_parent.name);
                    if keep {
                        let (sig_start, sig_end) = parsed_parent.lines_signature;
                        for idx in sig_start..=sig_end {
                            parent_indices.insert(idx);
                        }
                        keeps.push(*hashtree_key);
                    }
                }
            }
        }
    }
    (keeps, parent_indices)
}

impl CommandActions for CDeclarations {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        fn f_calculate_visibility(visible: bool, hit: bool) -> bool {
            visible || hit
        }

        // Build a dummy Vec<LineStatus> from hashtree entries
        // Each entry represents one declaration, with name as the searchable text
        // Maintain a parallel mapping of enumeration index to hashtree keys
        let hashtree_keys_ordered: Vec<usize> = hashtree
            .iter()
            .filter_map(|(key, parsed)| parsed.as_declarations().map(|_| *key))
            .collect();

        let vec_names: Vec<LineStatus> = hashtree_keys_ordered
            .iter()
            .map(|hashtree_key| {
                let parsed = hashtree[hashtree_key]
                    .as_declarations()
                    .expect("filtered declarations key should map to declarations");
                LineStatus {
                    line_number: *hashtree_key,
                    visible: false,
                    line: parsed.name.clone(),
                    ..Default::default()
                }
            })
            .collect();

        // Search on declaration names instead of file lines
        let (hit_indices, _) = search_vector(
            f_calculate_visibility,
            &self.searcher,
            vec_names,
            CommandVariant::CMore,
            &self.qualifier.searchqualifier,
        );

        // Convert hit indices back to hashtree keys
        let mut hit_hashtree_keys: Vec<usize> = hit_indices
            .iter()
            .filter_map(|idx| hashtree_keys_ordered.get(*idx).copied())
            .collect();

        if let Some(type_) = &self.qualifier.where_type {
            hit_hashtree_keys = filter_by_nodetype(type_, hashtree, hit_hashtree_keys);
        }

        // Build context indices from matched declarations' signature lines
        let mut flip_to_visibles = HashSet::new();
        let mut signature_starts: Vec<usize> = Vec::new();
        let mut signature_ends: Vec<usize> = Vec::new();

        let mut result_lines: Vec<LineStatus>;
        let parents_to_show: HashSet<usize>; // = HashSet::new();

        if *DEBUGGING {
            dbg_indices("&hit_hashtree_keys@345:", &hit_hashtree_keys);
            dbg!(&self.qualifier);
        }

        //filter on parent constraints given via `declare::foo/bar`
        if self.qualifier.parents.is_some() {
            (hit_hashtree_keys, parents_to_show) = filter_parent_where(
                hashtree,
                self.qualifier
                    .parent_matchers
                    .as_deref()
                    .expect("missing parent matchers for parent-qualified declaration command"),
                &hit_hashtree_keys,
            );
        } else {
            parents_to_show = HashSet::new();
        }

        //set visible=true on the signature lines of hits
        for hashtree_key in &hit_hashtree_keys {
            if let Some(parsed) = hashtree
                .get(hashtree_key)
                .and_then(|parsed| parsed.as_declarations())
            {
                if *DEBUGGING {
                    dbg!(&parsed);
                }

                let (sig_start, sig_end) = parsed.lines_signature;
                signature_starts.push(sig_start);
                signature_ends.push(sig_end);
                for idx in sig_start..=sig_end {
                    if idx < lines.len() {
                        flip_to_visibles.insert(idx);
                    }
                }
            } else {
                error!(
                    "{}",
                    user_messages::declarations_hashtree_key_not_found(*hashtree_key)
                );
            }
        }

        let saved = flip_to_visibles.clone();

        if self.qualifier.searchqualifier.before > 0 {
            vec_lines::bounded_extend(
                &mut flip_to_visibles,
                &signature_starts,
                -self.qualifier.searchqualifier.before,
                &lines,
            );
        }

        if self.qualifier.searchqualifier.after > 0 {
            vec_lines::bounded_extend(
                &mut flip_to_visibles,
                &signature_ends,
                self.qualifier.searchqualifier.after,
                &lines,
            );
        }

        // Apply visibility to original lines based on signature matches
        if *DEBUGGING {
            dbg!(&parents_to_show);
        }

        dbg_indices(
            "&parents_to_show@236:",
            &parents_to_show.clone().into_iter().collect(),
        );

        //show parents, when applicable
        result_lines = set_visibles(lines, &parents_to_show);
        //show
        result_lines = set_visibles(result_lines, &flip_to_visibles);

        if let Some(set_key) = &self.qualifier.searchqualifier.set_key {
            result_lines = update_meta_key(result_lines, &saved, set_key);
        }

        if self.qualifier.showbody {
            result_lines = show_body_from_hashtree(&hit_hashtree_keys, result_lines, hashtree)
        }
        if self.qualifier.searchqualifier.showowner {
            match self.qualifier.extension.as_str() {
                "py" | "rs" | "js" | "mjs" | "cjs" => {
                    result_lines =
                        show_owner_from_hashtree(&hit_hashtree_keys, result_lines, hashtree)
                }
                _ => {
                    warn!(
                        "{}",
                        user_messages::declarations_showowner_extension_unsupported()
                    );
                }
            }
        }
        result_lines
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}

#[cfg(test)]
mod tests {
    use super::{build_parent_matchers, filter_parent_where};
    use crate::CommandActions;
    use crate::base::{LineStatus, MetaInfo};
    use crate::common::{
        AppConfig, Parsed, ParsedDeclarationsData, reset_global_state_for_tests, set_global_config,
    };
    use crate::{Command, commands::declarations::from_declarations};
    use std::collections::HashMap;

    fn declaration(name: &str, signature_line: usize, parents: Vec<usize>) -> Parsed {
        Parsed::Declarations(ParsedDeclarationsData {
            name: name.to_string(),
            lines_signature: (signature_line, signature_line),
            parents,
            ..Default::default()
        })
    }

    #[test]
    fn filter_parent_where_uses_compiled_fixed_matchers() {
        let mut hashtree = HashMap::new();
        hashtree.insert(10, declaration("WidgetController", 1, vec![]));
        hashtree.insert(20, declaration("render", 3, vec![10]));
        hashtree.insert(30, declaration("skip_render", 5, vec![10]));

        let parent_matchers =
            build_parent_matchers(&Some(vec!["Widget,-skip".to_string()]), true, false)
                .expect("parent matcher should compile")
                .expect("parent matcher should exist");

        let hits = vec![20, 30];
        let (keeps, parent_indices) = filter_parent_where(&hashtree, &parent_matchers, &hits);

        assert_eq!(keeps, vec![20, 30]);
        assert!(parent_indices.contains(&1));
    }


    #[test]
    fn showowner_for_js_uses_parents_to_include_class_owner() {
        reset_global_state_for_tests();
        set_global_config(AppConfig {
            file_path: "sample.js".to_string(),
            ..Default::default()
        });

        let cmd = from_declarations("gree", "o", "d::gree::o");
        let Command::CDeclarations(cdecl) = cmd else {
            panic!("expected CDeclarations");
        };

        let mut hashtree = HashMap::new();
        hashtree.insert(
            0,
            Parsed::Declarations(ParsedDeclarationsData {
                name: "Greeter".to_string(),
                lines_signature: (0, 0),
                type_: "c".to_string(),
                ..Default::default()
            }),
        );
        hashtree.insert(
            1,
            Parsed::Declarations(ParsedDeclarationsData {
                name: "greet".to_string(),
                lines_signature: (1, 1),
                parents: vec![0],
                type_: "f".to_string(),
                ..Default::default()
            }),
        );

        let lines = vec!["class Greeter {", "    greet(name) {", "    }", "}"]
            .into_iter()
            .enumerate()
            .map(|(idx, line)| LineStatus {
                line_number: idx,
                visible: false,
                line: line.to_string(),
                meta: MetaInfo::default(),
            })
            .collect::<Vec<_>>();

        let result = cdecl.apply(lines, &hashtree);
        assert!(result[0].visible);
        assert!(result[1].visible);
    }
}
