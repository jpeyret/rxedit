use tree_sitter::{Node, Parser};

use crate::common::{ParsedDeclarationsData, ParsedImportsData};
use crate::constants::grammars;
use crate::constants::treesitter_;
use crate::languages;
use crate::treesitterparser::{Helper, LanguageHelper, LanguageSpecificConfig};
use crate::user_messages;

use crate::constants::DEBUGGING;

use log::warn;

static WANTED_DECLARES: std::sync::LazyLock<Vec<&'static str>> =
    std::sync::LazyLock::new(|| languages::phf_keys(&grammars::rust::PHF_WANTED_DECLARES));

static WANTED_IMPORTS: std::sync::LazyLock<Vec<&'static str>> =
    std::sync::LazyLock::new(|| languages::phf_keys(&grammars::rust::PHF_WANTED_IMPORTS));

#[derive(Debug, Clone, Copy, Default)]
/// Rust implementation of language parsing helpers.
pub struct HelperRust;

impl LanguageHelper for HelperRust {
    fn wanted_declares(&self) -> &'static [&'static str] {
        WANTED_DECLARES.as_slice()
    }

    fn wanted_imports(&self) -> &'static [&'static str] {
        WANTED_IMPORTS.as_slice()
    }

    fn parse_imports(&self, source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
        if *DEBUGGING {
            eprintln!("071:parse_imports:node={}", node.grammar_name());
            if node.start_position().row == node.end_position().row {
                let code = node.utf8_text(source_code).unwrap_or_default();
                eprintln!("071:parse_imports:code={code}");
            }
        }
        match node.kind() {
            "use_declaration" => parse_use_declarations(source_code, node),
            "mod_item" => parse_mod_item_import(source_code, node),
            _ => None,
        }
    }

    fn parse_declarations(
        &self,
        source_code: &[u8],
        node: Node<'_>,
        wanted_declares: &[&str],
    ) -> Option<ParsedDeclarationsData> {
        parse_declaration_rust(source_code, node, wanted_declares)
    }
}

fn node_text(source_code: &[u8], node: Node<'_>) -> Option<String> {
    std::str::from_utf8(&source_code[node.start_byte()..node.end_byte()])
        .ok()
        .map(|s| s.to_string())
}

fn build_imports(
    source_code: &[u8],
    node: Node<'_>,
    names: Vec<String>,
) -> Option<ParsedImportsData> {
    if names.is_empty() {
        return None;
    }

    Some(ParsedImportsData {
        body: node_text(source_code, node)?,
        names,
        start_line: node.start_position().row,
        end_line: node.end_position().row,
    })
}

fn normalize_segments(segments: Vec<String>) -> Vec<String> {
    segments
        .into_iter()
        .filter(|segment| !matches!(segment.as_str(), "crate" | "self" | "super"))
        .collect()
}

fn collect_path_segments(source_code: &[u8], node: Node<'_>) -> Vec<String> {
    match node.kind() {
        "identifier" | "self" | "super" | "crate" | "type_identifier" => {
            node_text(source_code, node).into_iter().collect()
        }
        "scoped_identifier" | "scoped_type_identifier" => {
            let mut result = node
                .child_by_field_name("path")
                .map(|path| collect_path_segments(source_code, path))
                .unwrap_or_default();

            if let Some(name) = node.child_by_field_name("name") {
                result.extend(collect_path_segments(source_code, name));
            }

            result
        }
        _ => Vec::new(),
    }
}

fn parse_use_declarations(source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
    if *DEBUGGING {
        eprintln!("071:104:parse_use_declarations node={node}");
    }

    let argument = node.child_by_field_name("argument")?;
    let raw_argument = node_text(source_code, argument)?;
    if raw_argument
        .split("::")
        .any(|segment| matches!(segment.trim(), "self" | "super"))
    {
        return None;
    }

    let mut names = match argument.kind() {
        "scoped_use_list" => argument
            .child_by_field_name("path")
            .map(|path| normalize_segments(collect_path_segments(source_code, path)))
            .unwrap_or_default(),
        "use_as_clause" => argument
            .child_by_field_name("path")
            .map(|path| normalize_segments(collect_path_segments(source_code, path)))
            .unwrap_or_default(),
        "use_wildcard" => {
            let mut cursor = argument.walk();
            let path = argument.named_children(&mut cursor).next();
            path.map(|path| normalize_segments(collect_path_segments(source_code, path)))
                .unwrap_or_default()
        }
        _ => {
            let mut segments = normalize_segments(collect_path_segments(source_code, argument));
            if segments.len() > 1 {
                segments.pop();
            }
            segments
        }
    };
    if *DEBUGGING {
        eprintln!("071:135:names{:?}", names);
    }
    names.retain(|name| !name.is_empty());
    build_imports(source_code, node, names)
}

fn parse_mod_item_import(source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
    if node.child_by_field_name("body").is_some() {
        return None;
    }

    let name = node.child_by_field_name("name")?;
    build_imports(source_code, node, vec![node_text(source_code, name)?])
}

fn get_parents(wanted: &[&str], node: Option<Node>) -> Vec<usize> {
    let mut result = Vec::new();
    if let Some(n) = node {
        if wanted.contains(&n.kind()) {
            result.push(n.start_position().row);
        }
        if let Some(parent) = n.parent() {
            result.extend(get_parents(wanted, Some(parent)));
        }
    }
    result
}

/// Rust-specific declarations parser
/// Handles the special case of `impl_item` which has no `name` field
fn parse_declaration_rust(
    source_code: &[u8],
    node: Node<'_>,
    wanted_declares: &[&str],
) -> Option<ParsedDeclarationsData> {
    // `impl_item` has no `name` field; synthesise one from `trait` and `type`.
    let name = if node.kind() == "impl_item" {
        let type_name = node
            .child_by_field_name("type")
            .and_then(|n| node_text(source_code, n))
            .unwrap_or_default();
        let trait_name = node
            .child_by_field_name("trait")
            .and_then(|n| node_text(source_code, n));
        match trait_name {
            Some(tr) => format!("{} for {}", tr, type_name),
            None => type_name,
        }
    } else {
        let name_node = match node.child_by_field_name(treesitter_::FIELD_NAME) {
            Some(name_node) => name_node,
            None => {
                let preview = node_text(source_code, node)
                    .unwrap_or_default()
                    .chars()
                    .take(20)
                    .collect::<String>();
                warn!(
                    "{}",
                    user_messages::treesitter_name_field_missing(node.kind(), &preview)
                );
                return None;
            }
        };
        node_text(source_code, name_node)?
    };

    let start_line = node.start_position().row;
    let lines_body = node
        .child_by_field_name(treesitter_::FIELD_BODY)
        .map(|body_node| (body_node.start_position().row, body_node.end_position().row));

    let signature_end_line = match lines_body {
        Some((body_start, _)) => match node.kind() {
            "function_definition" | "class_definition" => body_start.saturating_sub(1),
            _ => body_start,
        },
        None => start_line,
    };
    let end_line = node.end_position().row;

    let signature = node_text(source_code, node)?
        .replace('\n', " ")
        .trim()
        .to_string();
    let lines_signature = (start_line, signature_end_line);

    let parents = get_parents(wanted_declares, node.parent());

    Some(ParsedDeclarationsData {
        name,
        start_line,
        end_line,
        signature,
        lines_signature,
        lines_body,
        lines_doc: None,
        parents,
        lines_qualifier: None,
        type_: node.kind().to_string(),
    })
}

/// Builds parser and helper configuration for Rust sources.
pub fn build_config() -> Option<LanguageSpecificConfig> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).ok()?;

    Some(LanguageSpecificConfig {
        parser,
        helper: Helper::Rust(HelperRust),
    })
}
