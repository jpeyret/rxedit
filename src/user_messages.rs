//! stores big user-facing messages, esp. error messages.

/// Stable user-facing message codes.
pub mod codes {
    /// Error code for missing declarations hashtree key.
    pub const DECLARATIONS_HASHTREE_KEY_NOT_FOUND: &str = "RXE-ERR-0001";
    /// Warning code for unsupported showowner extension.
    pub const DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED: &str = "RXE-WARN-0001";
    /// Warning code for missing tree-sitter name field.
    pub const TREESITTER_NAME_FIELD_MISSING: &str = "RXE-WARN-0002";
}

/// Short message bodies shown to users.
pub mod body {
    /// Body template for missing declarations hashtree key.
    pub const DECLARATION_HASHTREE_KEY_NOT_FOUND: &str = "hashtree_key={} not found";
    /// Body for unsupported showowner extension.
    pub const DECLARATION_SHOWOWNER_EXTENSION_UNSUPPORTED: &str =
        "apply:showowner extension did not match supported languages";
    /// Body template for missing tree-sitter name field.
    pub const TREESITTER_NAME_FIELD_MISSING: &str =
        "Could not find name field for node kind: {} : {}";
}

/// Long-form details for logs and diagnostics.
pub mod details {
    /// Details for missing declarations hashtree key.
    pub const DECLARATIONS_HASHTREE_KEY_NOT_FOUND: &str = r#"
This error means a `declarations` hit referenced a tree-sitter key that was not found in the parsed declarations map.
This usually indicates declarations state got out of sync between parsing and filtering.
"#;
    /// Details for unsupported showowner extension.
    pub const DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED: &str = r#"
This warning means the programming language extension for the provided file does not have an rxedit tree-sitter grammar defined yet.
Basic commands like more/all/less still work, but syntax-aware features such as declarations and show owner are unavailable for that extension.
"#;
    /// Details for missing tree-sitter name field.
    pub const TREESITTER_NAME_FIELD_MISSING: &str = r#"
This warning means rxedit found a declarations node that does not expose the expected name field in the current grammar,
so that declarations is skipped when building the declaration index.
"#;
}

#[derive(Debug, Clone, Copy)]
/// Canonical definition for a user-facing message.
pub struct UserMessageDef {
    /// Internal message key.
    pub key: &'static str,
    /// Stable external code.
    pub code: &'static str,
    /// Short user-visible body text.
    pub body: &'static str,
    /// Longer diagnostic details.
    pub details: &'static str,
}

/// Message definition: declarations hashtree key not found.
pub const DECLARATIONS_HASHTREE_KEY_NOT_FOUND: UserMessageDef = UserMessageDef {
    key: "declarations_hashtree_key_not_found",
    code: codes::DECLARATIONS_HASHTREE_KEY_NOT_FOUND,
    body: body::DECLARATION_HASHTREE_KEY_NOT_FOUND,
    details: details::DECLARATIONS_HASHTREE_KEY_NOT_FOUND,
};

/// Message definition: showowner extension unsupported.
pub const DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED: UserMessageDef = UserMessageDef {
    key: "declarations_showowner_extension_unsupported",
    code: codes::DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED,
    body: body::DECLARATION_SHOWOWNER_EXTENSION_UNSUPPORTED,
    details: details::DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED,
};

/// Message definition: tree-sitter name field missing.
pub const TREESITTER_NAME_FIELD_MISSING: UserMessageDef = UserMessageDef {
    key: "treesitter_name_field_missing",
    code: codes::TREESITTER_NAME_FIELD_MISSING,
    body: body::TREESITTER_NAME_FIELD_MISSING,
    details: details::TREESITTER_NAME_FIELD_MISSING,
};

/// Registry of all message definitions.
pub const ALL: &[UserMessageDef] = &[
    DECLARATIONS_HASHTREE_KEY_NOT_FOUND,
    DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED,
    TREESITTER_NAME_FIELD_MISSING,
];

/// Builds the formatted missing-hashtree-key message.
pub fn declarations_hashtree_key_not_found(hashtree_key: usize) -> String {
    let body =
        body::DECLARATION_HASHTREE_KEY_NOT_FOUND.replacen("{}", &hashtree_key.to_string(), 1);
    format!("[{}] {}", codes::DECLARATIONS_HASHTREE_KEY_NOT_FOUND, body)
}

/// Builds the formatted unsupported-extension message.
pub fn declarations_showowner_extension_unsupported() -> String {
    format!(
        "[{}] {}",
        codes::DECLARATIONS_SHOWOWNER_EXTENSION_UNSUPPORTED,
        body::DECLARATION_SHOWOWNER_EXTENSION_UNSUPPORTED
    )
}

/// Builds the formatted missing-name-field message.
pub fn treesitter_name_field_missing(node_kind: &str, preview: &str) -> String {
    let body = body::TREESITTER_NAME_FIELD_MISSING
        .replacen("{}", node_kind, 1)
        .replacen("{}", preview, 1);
    format!("[{}] {}", codes::TREESITTER_NAME_FIELD_MISSING, body)
}
