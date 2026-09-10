// this IS NOT PART of the rxedit project, it is a copied sample of a current experimental zsh parser implementation, already usable by rxedit.

use abi_stable::StableAbi;
use abi_stable::std_types::{RString, RVec};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub mod language_api;
use language_api::{Parsed, ParsedDeclarationsData, ParsedImportsData};

const ABI_VERSION: u32 = 1;
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
const LANGUAGE_ID: &str = "zsh";

#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
pub struct ZshPluginMetadata {
    pub abi_version: u32,
    pub plugin_version: RString,
    pub language_id: RString,
    pub supports_declarations: bool,
    pub supports_imports: bool,
}

#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
pub struct ZshParseResponse {
    pub hashtree_json: RString,
    pub diagnostics_json: RString,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct ZshPluginApi {
    pub metadata: extern "C" fn() -> ZshPluginMetadata,
    pub parse_content_to_hashtree_json: extern "C" fn(source_code: RVec<u8>) -> ZshParseResponse,
    pub inspect_node_kinds_json: extern "C" fn(source_code: RVec<u8>) -> RString,
}

#[derive(Debug, Serialize)]
struct ParserDiagnostics {
    status: String,
    root_kind: Option<String>,
    node_count: Option<usize>,
    note: String,
}

#[derive(Debug, Serialize)]
struct NodeKindCount {
    kind: String,
    count: usize,
}

#[derive(Debug, Serialize)]
struct NodeKindSummary {
    status: String,
    root_kind: Option<String>,
    total_nodes: usize,
    distinct_kinds: usize,
    node_kinds: Vec<NodeKindCount>,
    note: String,
}

#[unsafe(no_mangle)]
pub extern "C" fn metadata() -> ZshPluginMetadata {
    ZshPluginMetadata {
        abi_version: ABI_VERSION,
        plugin_version: PLUGIN_VERSION.into(),
        language_id: LANGUAGE_ID.into(),
        supports_declarations: true,
        supports_imports: true,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn parse_content_to_hashtree_json(source_code: RVec<u8>) -> ZshParseResponse {
    let source_code: Vec<u8> = source_code.into();
    let hashtree = extract_declarations_hashtree(&source_code);

    // Phase 2 scaffold: this function intentionally parses and reports tree shape,
    // and currently only supports function_definition declarations.
    let diagnostics = match parse_zsh_source_debug(&source_code) {
        Ok((root_kind, node_count)) => ParserDiagnostics {
            status: "ok".to_string(),
            root_kind: Some(root_kind),
            node_count: Some(node_count),
            note: "Declarations are extracted from function_definition nodes and imports are extracted from source command statements.".to_string(),
        },
        Err(error) => ParserDiagnostics {
            status: "error".to_string(),
            root_kind: None,
            node_count: None,
            note: error,
        },
    };

    ZshParseResponse {
        hashtree_json: serde_json::to_string(&hashtree)
            .unwrap_or_else(|_| "{}".to_string())
            .into(),
        diagnostics_json: serde_json::to_string(&diagnostics)
            .unwrap_or_else(|_| "{\"status\":\"error\",\"note\":\"failed to serialize diagnostics\"}".to_string())
            .into(),
    }
}

fn extract_declarations_hashtree(source_code: &[u8]) -> HashMap<usize, Parsed> {
    let tree = match parse_zsh_tree(source_code) {
        Ok(tree) => tree,
        Err(_) => return HashMap::new(),
    };

    let source_text = String::from_utf8_lossy(source_code);
    let mut hashtree = HashMap::new();
    let mut cursor = tree.walk();

    fn walk(
        cursor: &mut tree_sitter::TreeCursor<'_>,
        source_text: &str,
        hashtree: &mut HashMap<usize, Parsed>,
    ) {
        let node = cursor.node();
        if node.kind() == "function_definition" {
            let declaration = build_function_declaration(source_text, node);
            hashtree.insert(declaration.start_line, Parsed::Declarations(declaration));
        } else if let Some(imports) = build_source_import(source_text, node) {
            hashtree.insert(imports.start_line, Parsed::Imports(imports));
        }

        if cursor.goto_first_child() {
            loop {
                walk(cursor, source_text, hashtree);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    walk(&mut cursor, &source_text, &mut hashtree);
    hashtree
}

fn build_source_import(source_text: &str, node: tree_sitter::Node<'_>) -> Option<ParsedImportsData> {
    if node.kind() != "command" {
        return None;
    }

    let body = source_text
        .get(node.start_byte()..node.end_byte())?
        .trim()
        .to_string();
    let mut parts = body.split_whitespace();
    let command = parts.next()?;
    if command != "source" {
        return None;
    }

    let name = parts
        .next()?
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string();
    if name.is_empty() {
        return None;
    }

    Some(ParsedImportsData {
        body,
        names: vec![name],
        start_line: node.start_position().row,
        end_line: node.end_position().row,
    })
}

fn build_function_declaration(source_text: &str, node: tree_sitter::Node<'_>) -> ParsedDeclarationsData {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;
    let signature_line = source_text
        .lines()
        .nth(start_line)
        .unwrap_or_default()
        .trim()
        .to_string();

    let name = extract_function_name(&signature_line)
        .unwrap_or_else(|| "<anonymous>".to_string());

    ParsedDeclarationsData {
        name,
        start_line,
        end_line,
        signature: signature_line,
        lines_signature: (start_line, start_line),
        lines_body: if end_line > start_line {
            Some((start_line + 1, end_line))
        } else {
            None
        },
        lines_doc: None,
        parents: Vec::new(),
        lines_qualifier: None,
        // `f` maps to function for current where_type shorthand.
        type_: "f".to_string(),
    }
}

fn extract_function_name(signature_line: &str) -> Option<String> {
    let function_keyword = Regex::new(r"^function\s+([A-Za-z_][A-Za-z0-9_]*)").ok()?;
    if let Some(captures) = function_keyword.captures(signature_line) {
        return captures.get(1).map(|m| m.as_str().to_string());
    }

    let bare_form = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)").ok()?;
    bare_form
        .captures(signature_line)
        .and_then(|captures| captures.get(1).map(|m| m.as_str().to_string()))
}

#[unsafe(no_mangle)]
pub extern "C" fn inspect_node_kinds_json(source_code: RVec<u8>) -> RString {
    let source_code: Vec<u8> = source_code.into();
    let summary = match parse_zsh_tree(&source_code) {
        Ok(tree) => {
            let root = tree.root_node();
            let counts = collect_node_kinds(root);
            let total_nodes = counts.values().sum();
            let node_kinds = counts
                .into_iter()
                .map(|(kind, count)| NodeKindCount { kind, count })
                .collect::<Vec<_>>();

            NodeKindSummary {
                status: "ok".to_string(),
                root_kind: Some(root.kind().to_string()),
                total_nodes,
                distinct_kinds: node_kinds.len(),
                node_kinds,
                note: "Use these kinds to choose declaration candidates for Phase 2 parser implementation.".to_string(),
            }
        }
        Err(error) => NodeKindSummary {
            status: "error".to_string(),
            root_kind: None,
            total_nodes: 0,
            distinct_kinds: 0,
            node_kinds: Vec::new(),
            note: error,
        },
    };

    serde_json::to_string(&summary)
        .unwrap_or_else(|_| "{\"status\":\"error\",\"note\":\"failed to serialize node-kind summary\"}".to_string())
        .into()
}

fn parse_zsh_source_debug(source_code: &[u8]) -> Result<(String, usize), String> {
    let tree = parse_zsh_tree(source_code)?;
    let root = tree.root_node();
    let node_count = count_nodes(root);
    Ok((root.kind().to_string(), node_count))
}

fn parse_zsh_tree(source_code: &[u8]) -> Result<tree_sitter::Tree, String> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_zsh::LANGUAGE.into())
        .map_err(|err| format!("failed to set zsh grammar: {err}"))?;

    parser
        .parse(source_code, None)
        .ok_or_else(|| "tree-sitter returned no parse tree".to_string())
}

fn count_nodes(root: tree_sitter::Node<'_>) -> usize {
    let mut count = 1;
    let mut cursor = root.walk();

    fn walk(cursor: &mut tree_sitter::TreeCursor<'_>, count: &mut usize) {
        if cursor.goto_first_child() {
            loop {
                *count += 1;
                walk(cursor, count);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    walk(&mut cursor, &mut count);
    count
}

fn collect_node_kinds(root: tree_sitter::Node<'_>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let mut cursor = root.walk();

    fn walk(cursor: &mut tree_sitter::TreeCursor<'_>, counts: &mut BTreeMap<String, usize>) {
        let kind = cursor.node().kind().to_string();
        *counts.entry(kind).or_insert(0) += 1;

        if cursor.goto_first_child() {
            loop {
                walk(cursor, counts);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    walk(&mut cursor, &mut counts);
    counts
}

#[unsafe(no_mangle)]
pub extern "C" fn get_plugin_api() -> ZshPluginApi {
    ZshPluginApi {
        metadata,
        parse_content_to_hashtree_json,
        inspect_node_kinds_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_flags_imports_as_unsupported() {
        let info = metadata();
        assert_eq!(info.language_id.as_str(), "zsh");
        assert!(info.supports_declarations);
        assert!(info.supports_imports);
    }

    #[test]
    fn parser_debug_runs_on_basic_input() {
        let input = b"function greet() {\n  echo hi\n}\n";
        let (root_kind, node_count) = parse_zsh_source_debug(input).expect("expected parse tree");
        assert!(!root_kind.is_empty());
        assert!(node_count > 0);
    }

    #[test]
    fn node_kind_inspection_returns_json_summary() {
        let input = b"function greet() {\n  echo hi\n}\n";
        let summary = inspect_node_kinds_json(RVec::from(input.to_vec()));
        let summary = summary.into_string();
        assert!(summary.contains("\"status\":\"ok\""));
        assert!(summary.contains("node_kinds"));
    }

    #[test]
    fn extracts_function_definition_declaration() {
        let input = b"function greet() {\n  echo hi\n}\n";
        let response = parse_content_to_hashtree_json(RVec::from(input.to_vec()));
        let json = response.hashtree_json.into_string();
        assert!(json.contains("greet"));
        assert!(json.contains("\"type_\":\"f\""));
    }

    #[test]
    fn extracts_bare_zsh_function_syntax_declarations() {
        let input = b"llhelp(){\n  echo hi\n}\n\nlhello(){\n  echo hello world\n}\n";
        let response = parse_content_to_hashtree_json(RVec::from(input.to_vec()));
        let json = response.hashtree_json.into_string();
        assert!(json.contains("llhelp"));
        assert!(json.contains("lhello"));
        assert!(json.contains("\"type_\":\"f\""));
    }

    #[test]
    fn extracts_source_statement_imports() {
        let input = b"source ./env.zsh\nfunction greet() {\n  echo hi\n}\n";
        let response = parse_content_to_hashtree_json(RVec::from(input.to_vec()));
        let json = response.hashtree_json.into_string();
        assert!(json.contains("\"Imports\""));
        assert!(json.contains("./env.zsh"));
    }
}
