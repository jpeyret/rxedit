//! Shared language plugin API used by the host and by future .so-backed language support modules.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Parsed declaration metadata extracted from source.
pub struct ParsedDeclarationsData {
    /// Declaration name.
    pub name: String,
    /// Start line of the declaration.
    pub start_line: usize,
    /// End line of the declaration.
    pub end_line: usize,
    /// Signature text for the declaration.
    pub signature: String,
    /// Line range for the signature.
    pub lines_signature: (usize, usize),
    /// Optional line range for declaration body.
    pub lines_body: Option<(usize, usize)>,
    /// Optional line range for declaration docs.
    pub lines_doc: Option<(usize, usize)>,
    /// Parent declaration start lines.
    pub parents: Vec<usize>,
    // this is for things like python @ decorators or rust #[derive ...]  !!!TODO!!! not done, yet
    /// Optional line range for qualifiers/attributes.
    pub lines_qualifier: Option<(usize, usize)>,
    /// Declaration type shorthand.
    pub type_: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Parsed import metadata extracted from source.
pub struct ParsedImportsData {
    /// Full import text.
    pub body: String,
    /// Imported symbol names.
    pub names: Vec<String>,
    /// Start line of the import.
    pub start_line: usize,
    /// End line of the import.
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Parsed node kind stored in parse hashtrees.
pub enum Parsed {
    /// Parsed declaration entry.
    Declarations(ParsedDeclarationsData),
    /// Parsed import entry.
    Imports(ParsedImportsData),
}

impl Parsed {
    /// Returns the starting line for this parsed item.
    pub fn start_line(&self) -> usize {
        match self {
            Self::Declarations(parsed) => parsed.start_line,
            Self::Imports(parsed) => parsed.start_line,
        }
    }

    /// Returns declaration data when this item is a declaration.
    pub fn as_declarations(&self) -> Option<&ParsedDeclarationsData> {
        match self {
            Self::Declarations(parsed) => Some(parsed),
            Self::Imports(_) => None,
        }
    }

    /// Returns import data when this item is an import.
    pub fn as_imports(&self) -> Option<&ParsedImportsData> {
        match self {
            Self::Declarations(_) => None,
            Self::Imports(parsed) => Some(parsed),
        }
    }
}

/// Default no-op implementation used as the host-defined fallback for plugin authors.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultLanguageHelper;

impl LanguageHelper for DefaultLanguageHelper {
    fn wanted_declares(&self) -> &'static [&'static str] {
        &[]
    }

    fn wanted_imports(&self) -> &'static [&'static str] {
        &[]
    }

    fn parse_imports(&self, _source_code: &[u8], _node: Node<'_>) -> Option<ParsedImportsData> {
        None
    }

    fn parse_declarations(
        &self,
        _source_code: &[u8],
        _node: Node<'_>,
        _wanted_declares: &[&str],
    ) -> Option<ParsedDeclarationsData> {
        None
    }
}

/// Shared language API used by the host and by future plugin implementations.
pub trait LanguageHelper {
    /// Returns node kinds that may contain declarations.
    fn wanted_declares(&self) -> &'static [&'static str];
    /// Returns node kinds that may contain imports.
    fn wanted_imports(&self) -> &'static [&'static str];
    /// Parses an imports node into normalized imports data.
    fn parse_imports(&self, source_code: &[u8], node: Node<'_>) -> Option<ParsedImportsData>;
    /// Parses a declaration node into normalized declaration data.
    fn parse_declarations(
        &self,
        source_code: &[u8],
        node: Node<'_>,
        wanted_declares: &[&str],
    ) -> Option<ParsedDeclarationsData>;

    /// Plugin-facing boundary for source bytes to the normalized hashtree representation.
    ///
    /// The default implementation is intentionally conservative: this is the stable
    /// contract that future plugin crates can override, without depending on
    /// tree-sitter node lifetimes across the ABI boundary.
    fn parse_content_to_hashtree(&self, _source_code: &[u8]) -> HashMap<usize, Parsed> {
        HashMap::new()
    }
}
