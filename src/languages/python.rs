use tree_sitter::{Node, Parser};

use crate::common::{ParsedDeclarationsData, ParsedImportsData};
use crate::constants::grammars;
use crate::constants::treesitter_;
use crate::languages;
use crate::treesitterparser::{Helper, LanguageHelper, LanguageSpecificConfig};
use crate::user_messages;

use log::warn;

static WANTED_DECLARES: std::sync::LazyLock<Vec<&'static str>> =
    std::sync::LazyLock::new(|| languages::phf_keys(&grammars::python::PHF_WANTED_DECLARES));

static WANTED_IMPORTS: std::sync::LazyLock<Vec<&'static str>> =
    std::sync::LazyLock::new(|| languages::phf_keys(&grammars::python::PHF_WANTED_IMPORTS));

#[derive(Debug, Clone, Copy, Default)]
/// Python implementation of language parsing helpers.
pub struct HelperPython;

impl LanguageHelper for HelperPython {
    fn wanted_declares(&self) -> &'static [&'static str] {
        WANTED_DECLARES.as_slice()
    }

    fn wanted_imports(&self) -> &'static [&'static str] {
        WANTED_IMPORTS.as_slice()
    }

    fn parse_imports(&self, source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
        match node.kind() {
            "import_statement" => parse_import_statement(source_code, node),
            "import_from_statement" => parse_import_from_statement(source_code, node),
            _ => None,
        }
    }

    fn parse_declarations(
        &self,
        source_code: &[u8],
        node: Node<'_>,
        wanted_declares: &[&str],
    ) -> Option<ParsedDeclarationsData> {
        parse_declaration_python(source_code, node, wanted_declares)
    }
}

fn node_text(source_code: &[u8], node: Node<'_>) -> Option<String> {
    std::str::from_utf8(&source_code[node.start_byte()..node.end_byte()])
        .ok()
        .map(|s| s.to_string())
}

fn split_dotted_names(value: &str) -> Vec<String> {
    value
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
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

fn parse_import_statement(source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
    let mut cursor = node.walk();
    let mut names = Vec::new();

    for child in node.named_children(&mut cursor) {
        if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
            let name_node = if child.kind() == "aliased_import" {
                child.child_by_field_name("name")?
            } else {
                child
            };
            names.extend(split_dotted_names(&node_text(source_code, name_node)?));
        }
    }
    build_imports(source_code, node, names)
}

fn parse_import_from_statement(source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
    let module_name = node.child_by_field_name("module_name")?;
    let names = split_dotted_names(&node_text(source_code, module_name)?);
    build_imports(source_code, node, names)
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

/// Python-specific declarations parser
/// Handles the special case of comments appearing between signature and body
fn parse_declaration_python(
    source_code: &[u8],
    node: Node<'_>,
    wanted_declares: &[&str],
) -> Option<ParsedDeclarationsData> {
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
    let name = node_text(source_code, name_node)?;
    let start_line = node.start_position().row;

    let lines_body = node
        .child_by_field_name(treesitter_::FIELD_BODY)
        .map(|body_node| {
            // at least in python, a comment sits between the identifier/parameters and the body
            let end = body_node.end_position().row;

            let mut node = body_node;
            let startnode = loop {
                if let Some(prev) = &node.prev_named_sibling() {
                    if prev.grammar_name() == treesitter_::FIELD_COMMENT {
                        node = *prev;
                        continue;
                    } else {
                        break node;
                    }
                }
                break node;
            };
            let start = startnode.start_position().row;
            (start, end)
        });

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

/// Builds parser and helper configuration for Python sources.
pub fn build_config() -> Option<LanguageSpecificConfig> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).ok()?;

    Some(LanguageSpecificConfig {
        parser,
        helper: Helper::Python(HelperPython),
    })
}
