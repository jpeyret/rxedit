//! Utility functions to integrate with tree-sitter.
//! This file doesn't know particular language grammars.

#![allow(clippy::upper_case_acronyms)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config_utilities;

use abi_stable::StableAbi;
use abi_stable::std_types::{RString, RVec};
use libloading::Library;
use tree_sitter::{Node, Parser, Tree};

use crate::language_api::{Parsed, ParsedDeclarationsData, ParsedImportsData};
use crate::languages;

pub use crate::language_api::LanguageHelper;

const HOST_ABI_VERSION: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
struct ZshPluginMetadata {
    pub abi_version: u32,
    pub plugin_version: RString,
    pub language_id: RString,
    pub supports_declarations: bool,
    pub supports_imports: bool,
}

#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
struct ZshParseResponse {
    pub hashtree_json: RString,
    pub diagnostics_json: RString,
}

#[repr(C)]
#[derive(StableAbi)]
struct ZshPluginApi {
    pub metadata: extern "C" fn() -> ZshPluginMetadata,
    pub parse_content_to_hashtree_json: extern "C" fn(source_code: RVec<u8>) -> ZshParseResponse,
    pub inspect_node_kinds_json: extern "C" fn(source_code: RVec<u8>) -> RString,
}

fn parse_extension_language_mapping_from_toml(contents: &str) -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    let Ok(doc) = contents.parse::<toml::Table>() else {
        return mappings;
    };

    let Some(tree_sitter) = doc.get("tree_sitter").and_then(|value| value.as_table()) else {
        return mappings;
    };
    let Some(extensions) = tree_sitter.get("extensions").and_then(|value| value.as_table()) else {
        return mappings;
    };

    for (extension, language) in extensions {
        let Some(language_name) = language.as_str() else {
            continue;
        };
        mappings.insert(extension.trim().to_ascii_lowercase(), language_name.trim().to_string());
    }

    mappings
}


fn read_extension_language_mapping() -> HashMap<String, String> {
    let Ok(Some(contents)) = config_utilities::read_config_file_contents() else {
        return HashMap::new();
    };

    parse_extension_language_mapping_from_toml(&contents)
}

fn resolve_extension_language_id(extension: &str) -> Option<String> {
    let normalized = extension.to_ascii_lowercase();

    if let Some(language) = read_extension_language_mapping().get(&normalized) {
        return Some(language.clone());
    }

    match normalized.as_str() {
        "py" => Some("python".to_string()),
        "rs" => Some("rust".to_string()),
        "js" | "mjs" | "cjs" => Some("js".to_string()),
        "sh" | "bash" | "zsh" => Some("zsh".to_string()),
        _ => None,
    }
}

fn plugin_library_filename(language_id: &str) -> String {
    let ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(windows) {
        "dll"
    } else {
        "so"
    };

    format!("librxedit_language_{language_id}.{ext}")
}

fn resolve_plugin_library_path(language_id: &str) -> Option<PathBuf> {
    let candidate_name = plugin_library_filename(language_id);

    for directory in config_utilities::plugin_library_search_paths() {
        let candidate = directory.join(&candidate_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn normalize_hashtree_to_zero_based(_hashtree: &mut HashMap<usize, Parsed>) {
    // Internal indices stay zero-based. Human-facing conversions to 1-based line numbers
    // happen only at boundaries such as CLI printing and line selection parsing.
}

fn parse_language_plugin_hashtree(source_code: &[u8], language_id: &str) -> HashMap<usize, Parsed> {
    let library_path = match resolve_plugin_library_path(language_id) {
        Some(path) => path,
        None => return HashMap::new(),
    };

    let library = match unsafe { Library::new(&library_path) } {
        Ok(library) => library,
        Err(_) => return HashMap::new(),
    };

    let get_plugin_api: libloading::Symbol<unsafe extern "C" fn() -> ZshPluginApi> =
        match unsafe { library.get(b"get_plugin_api") } {
            Ok(symbol) => symbol,
            Err(_) => match unsafe { library.get(b"_get_plugin_api") } {
                Ok(symbol) => symbol,
                Err(_) => return HashMap::new(),
            },
        };

    let api = unsafe { get_plugin_api() };
    let metadata = (api.metadata)();
    if metadata.abi_version != HOST_ABI_VERSION || metadata.language_id.as_str() != language_id {
        return HashMap::new();
    }
    if !metadata.supports_declarations {
        return HashMap::new();
    }

    let response = (api.parse_content_to_hashtree_json)(RVec::from(source_code.to_vec()));
    let payload = response.hashtree_json.into_string();
    let mut parsed = match serde_json::from_str::<HashMap<usize, Parsed>>(&payload) {
        Ok(parsed) => parsed,
        Err(_) => return HashMap::new(),
    };
    normalize_hashtree_to_zero_based(&mut parsed);
    parsed
}

#[derive(Debug, Clone, Copy)]
/// Dispatches to the active language helper.
pub enum Helper {
    /// Python language helper.
    Python(languages::python::HelperPython),
    /// Rust language helper.
    Rust(languages::rust::HelperRust),
}

impl Helper {
    /// Returns declaration node kinds for the selected helper.
    pub fn wanted_declares(&self) -> &'static [&'static str] {
        match self {
            Self::Python(helper) => helper.wanted_declares(),
            Self::Rust(helper) => helper.wanted_declares(),
        }
    }

    /// Returns import node kinds for the selected helper.
    pub fn wanted_imports(&self) -> &'static [&'static str] {
        match self {
            Self::Python(helper) => helper.wanted_imports(),
            Self::Rust(helper) => helper.wanted_imports(),
        }
    }

    /// Parses imports using the selected helper.
    pub fn parse_imports(&self, source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
        match self {
            Self::Python(helper) => helper.parse_imports(source_code, node),
            Self::Rust(helper) => helper.parse_imports(source_code, node),
        }
    }

    /// Parses declarations using the selected helper.
    pub fn parse_declarations(
        &self,
        source_code: &[u8],
        node: Node<'_>,
        wanted_declares: &[&str],
    ) -> Option<ParsedDeclarationsData> {
        match self {
            Self::Python(helper) => helper.parse_declarations(source_code, node, wanted_declares),
            Self::Rust(helper) => helper.parse_declarations(source_code, node, wanted_declares),
        }
    }
}

impl LanguageHelper for Helper {
    fn wanted_declares(&self) -> &'static [&'static str] {
        self.wanted_declares()
    }

    fn wanted_imports(&self) -> &'static [&'static str] {
        self.wanted_imports()
    }

    fn parse_imports(&self, source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData> {
        self.parse_imports(source_code, node)
    }

    fn parse_declarations(
        &self,
        source_code: &[u8],
        node: Node<'_>,
        wanted_declares: &[&str],
    ) -> Option<ParsedDeclarationsData> {
        self.parse_declarations(source_code, node, wanted_declares)
    }

    fn parse_content_to_hashtree(&self, source_code: &[u8]) -> std::collections::HashMap<usize, Parsed> {
        match self {
            Self::Python(_) => {
                let mut config = languages::python::build_config().unwrap_or_else(|| {
                    panic!("python language helper could not build a parser")
                });
                parse_content_to_hashtree_internal(&mut config, source_code)
            }
            Self::Rust(_) => {
                let mut config = languages::rust::build_config().unwrap_or_else(|| {
                    panic!("rust language helper could not build a parser")
                });
                parse_content_to_hashtree_internal(&mut config, source_code)
            }
        }
    }
}

/// Parser and helper selected for a specific file extension.
pub struct LanguageSpecificConfig {
    /// Tree-sitter parser configured for the language.
    pub parser: Parser,
    /// Language helper used to extract parsed data.
    pub helper: Helper,
}

impl LanguageSpecificConfig {
    /// Builds a language config from a file extension.
    pub fn factory(extension: &str) -> Option<Self> {
        match extension {
            "py" => languages::python::build_config(),
            "rs" => languages::rust::build_config(),
            _ => None,
        }
    }
}

/// Reads a source file as raw bytes.
pub fn read_source_file(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

/// Parses a source file into a tree-sitter syntax tree.
pub fn parse_source_file(path: impl AsRef<Path>, extension: &str) -> Result<Tree, String> {
    let source_code = read_source_file(path).map_err(|e| e.to_string())?;
    let mut config = LanguageSpecificConfig::factory(extension)
        .ok_or_else(|| format!("unsupported extension: {}", extension))?;

    config
        .parser
        .parse(&source_code, None)
        .ok_or_else(|| "tree-sitter parse returned no tree".to_string())
}

fn parse_content_to_hashtree_internal(
    config: &mut LanguageSpecificConfig,
    source_code: &[u8],
) -> HashMap<usize, Parsed> {
    let tree = match config.parser.parse(source_code, None) {
        Some(tree) => tree,
        None => return HashMap::new(),
    };

    let mut hashtree: HashMap<usize, Parsed> = HashMap::new();
    let mut cursor = tree.walk();
    let mut visited_children = false;

    loop {
        if !visited_children {
            let node = cursor.node();
            let wanted_declares = config.helper.wanted_declares();
            let wanted_imports = config.helper.wanted_imports();
            let mut inserted = false;

            if wanted_imports.contains(&node.kind())
                && let Some(parsed) = config.helper.parse_imports(source_code, node)
            {
                hashtree.insert(parsed.start_line, Parsed::Imports(parsed));
                inserted = true;
            }

            if !inserted
                && wanted_declares.contains(&node.kind())
                && let Some(parsed) =
                    config
                        .helper
                        .parse_declarations(source_code, node, wanted_declares)
            {
                hashtree.insert(parsed.start_line, Parsed::Declarations(parsed));
            }

            if !cursor.goto_first_child() {
                visited_children = true;
            }
        } else if cursor.goto_next_sibling() {
            visited_children = false;
        } else if !cursor.goto_parent() {
            break;
        }
    }
    hashtree
}

/// Parses source bytes into a line-keyed parsed hashtree.
pub fn supports_extension_for_parsing(extension: &str) -> bool {
    if LanguageSpecificConfig::factory(extension).is_some() {
        return true;
    }

    resolve_extension_language_id(extension)
        .and_then(|language_id| resolve_plugin_library_path(&language_id))
        .is_some()
}

/// parses code via tree-sitter to return the locations of Nodes of special interest
pub fn parse_content_to_hashtree(source_code: &[u8], extension: &str) -> HashMap<usize, Parsed> {
    if let Some(mut config) = LanguageSpecificConfig::factory(extension) {
        return parse_content_to_hashtree_internal(&mut config, source_code);
    }

    match resolve_extension_language_id(extension) {
        Some(language_id) => parse_language_plugin_hashtree(source_code, &language_id),
        None => HashMap::new(),
    }
}

/// Serializes the parsed hashtree as JSON for debugging and host/plugin inspection.
pub fn parse_content_to_hashtree_json(source_code: &[u8], extension: &str) -> String {
    let hashtree = parse_content_to_hashtree(source_code, extension);
    serde_json::to_string_pretty(&hashtree).unwrap_or_else(|_| "{}".to_string())
}

/// Reads and parses a source file into a line-keyed parsed hashtree.
pub fn parse_source_file_to_hashtree(path: impl AsRef<Path>) -> HashMap<usize, Parsed> {
    let path_ref = path.as_ref();
    let extension = match path_ref.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ext,
        None => return HashMap::new(),
    };

    let source_code = match read_source_file(path_ref) {
        Ok(content) => content,
        Err(_) => return HashMap::new(),
    };

    parse_content_to_hashtree(&source_code, extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_content_to_hashtree_supports_shell_extension_via_language_plugin() {
        let source = b"function greet() {\n  echo hi\n}\n";
        let hashtree = parse_content_to_hashtree(source, "sh");
        assert!(!hashtree.is_empty());
        assert!(hashtree.values().any(|parsed| parsed.as_declarations().is_some()));
        assert!(hashtree
            .values()
            .filter_map(|parsed| parsed.as_declarations())
            .any(|decl| decl.name == "greet"));
    }


    #[test]
    fn config_mapping_supports_extension_to_language() {
        let config = r#"
[plugins]
directory = "~/.config/rxedit/plugins"

[tree_sitter.extensions]
sh = "zsh"
"#;

        let mappings = parse_extension_language_mapping_from_toml(config);
        assert_eq!(mappings.get("sh"), Some(&"zsh".to_string()));
        // assert_eq!(resolve_extension_language_id_from_toml(config, "sh"), Some("zsh".to_string()));
    }

    #[test]
    fn plugin_discovery_marks_shell_extension_as_supported() {
        assert!(supports_extension_for_parsing("sh"));
    }

    #[test]
    fn plugin_library_filename_uses_host_os_extension() {
        let name = plugin_library_filename("zsh");

        #[cfg(target_os = "macos")]
        assert_eq!(name, "librxedit_language_zsh.dylib");

        #[cfg(windows)]
        assert_eq!(name, "librxedit_language_zsh.dll");

        #[cfg(all(not(target_os = "macos"), not(windows)))]
        assert_eq!(name, "librxedit_language_zsh.so");
    }

    #[test]
    fn parse_content_to_hashtree_json_supports_shell_extension_via_language_plugin() {
        let source = b"function greet() {\n  echo hi\n}\n";
        let json = parse_content_to_hashtree_json(source, "sh");
        assert!(json.contains("\"0\":"));
        assert!(json.contains("\"start_line\": 0"));
        assert!(json.contains("\"lines_signature\": ["));
        assert!(json.contains("greet"));
    }

    #[test]
    fn parse_content_to_hashtree_json_keeps_zero_based_lines_for_third_line_function() {
        let source = b"\n\nllhelp(){\n  echo hi\n}\n";
        let json = parse_content_to_hashtree_json(source, "sh");
        assert!(json.contains("\"start_line\": 2"));
        assert!(json.contains("\"lines_signature\": ["));
        assert!(json.contains("\"lines_body\": ["));
        assert!(json.contains("llhelp"));
    }

    #[test]
    fn parse_content_to_hashtree_json_supports_js_imports_and_declarations_via_language_plugin() {
        let source = b"import path from \"path\";\nconst fs = require(\"fs\");\n\nclass Greeter {\n    constructor(prefix) {\n        this.prefix = prefix;\n    }\n\n    greet(name) {\n        return `${this.prefix} ${name}`;\n    }\n\n    static version() {\n        return \"1.0\";\n    }\n}\n\nfunction add(a, b) {\n    return a + b;\n}\n\nfunction formatFileName(name) {\n    return path.join(\"tmp\", name);\n}\n";

        if !supports_extension_for_parsing("js") {
            return;
        }

        let json = parse_content_to_hashtree_json(source, "js");
        assert!(json.contains("\"Imports\""));
        assert!(json.contains("\"path\""));
        assert!(json.contains("\"fs\""));

        assert!(json.contains("\"name\": \"Greeter\""));
        assert!(json.contains("\"type_\": \"c\""));

        for expected_fn_name in [
            "constructor",
            "greet",
            "version",
            "add",
            "formatFileName",
        ] {
            assert!(json.contains(expected_fn_name));
        }
    }
}
