//! Tracks constants for rxedit.
//! Notice:  as much as possible, user-facing constants
//! are NOT hardcoded here.  Instead they are loaded from file
//! `defaultconfig.toml`.  This means two things.  First, a scripting
//! language like Python can easily introspect definitions.
//! More importantly, `~/.config/rxedit/config.toml` can be used to
//! override those constants, which is intended to allow easier experimentation
//! to optimize DSL definition.  And also overrides just out of user preferences.
//! Either way, it's up to the user to keep the DSL coherent, nothing keeps you
//! from using say `A` for two different flags, which could let to problems.

#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use once_cell::sync::Lazy;
use regex::Regex;

/// command long and shortcodes
pub mod command_prefix {
    /// Long command: macro.
    pub const MACROS: &str = "macro";
    /// Long command: lines.
    pub const LINES: &str = "lines";
    /// Long command: more.
    pub const MORE: &str = "more";
    /// Long command: less.
    pub const LESS: &str = "less";
    /// Long command: all.
    pub const ALL: &str = "all";
    /// Long command: change.
    pub const CHANGE: &str = "change";
    /// Long command: declarations.
    pub const DECLARATIONS: &str = "declarations";
    /// Long command: imports.
    pub const IMPORTS: &str = "imports";
    /// Long command: and.
    pub const AND: &str = "and";
    /// Long command: output.
    pub const OUTPUT: &str = "output";
    /// Long command: delete.
    pub const DELETE: &str = "delete";
    /// Long command: invert.
    pub const INVERT: &str = "invert";
    /// Long command: explain.
    pub const EXPLAIN: &str = "explain";
    /// Long command: prepend.
    pub const PREPEND: &str = "prepend";
    /// Long command: append.
    pub const APPEND: &str = "append";

    /// Short command: more.
    pub const MORE1: &str = "m";
    /// Short command: all.
    pub const ALL1: &str = "a";
    /// Short command: less.
    pub const LESS1: &str = "l";
    /// Short command: change.
    pub const CHANGE1: &str = "c";
    /// Short command: declarations.
    pub const DECLARATIONS1: &str = "d";
    /// Short command: imports.
    pub const IMPORTS1: &str = "i";
    /// Short command: and.
    pub const AND1: &str = "n";
    /// Short command: output.
    pub const OUTPUT1: &str = "o";
    /// Short command: delete.
    pub const DELETE1: &str = "del";
    /// Short command: invert.
    pub const INVERT1: &str = "inv";
    /// Short command: prepend.
    pub const PREPEND1: &str = "pre";
    /// Short command: append.
    pub const APPEND1: &str = "app";
}

/// Tree-sitter field names used by grammar helpers.
pub mod treesitter_ {
    /// Field name for identifier names.
    pub const FIELD_NAME: &str = "name";
    /// Field name for body nodes.
    pub const FIELD_BODY: &str = "body";
    /// Field name for comment nodes.
    pub const FIELD_COMMENT: &str = "comment";
}

#[derive(PartialEq, Debug)]
/// Trim direction used by text operations.
pub enum Direction {
    /// Trim from the start.
    LEADING,
    /// Trim from the end.
    TRAILING,
    /// Trim from both ends.
    BOTH,
}

/// Command argument separator token.
pub const SEP: &str = "::";

/// defines what counts as a declation for each language.  should really be under the grammar.
pub mod declarations {
    /// Regex for Rust declaration-like lines.
    pub const RUST: &str = r"\bimpl |\bfn |\benum |\bstruct |\bmod |\btrait ";
    /// Regex for Python declaration-like lines.
    pub const PYTHON: &str = r"^ *class |^ *def |^ *@";
}

/// Environment variable names used by rxedit.
pub mod env_vars {
    /// Env var for line spacer customization.
    pub const SPACER: &str = "rxedit_spacer";
    /// Env var for line number display mode.
    pub const LINE_NUMBER: &str = "rxedit_line_number";
    /// Env var to toggle grep shortcodes.
    pub const GREP_SHORTCODES: &str = "rxedit_grep_shortcodes";
    /// Env var to override plugin directory for language parsers.
    pub const PLUGINS_DIRECTORY: &str = "rxedit_plugins_directory";
}

///Tracks language constructs relevant to CDeclarations and CImports commands.
pub mod grammars {

    /// Rust grammar node maps.
    pub mod rust {
        /// Rust node kinds treated as declarations.
        pub static PHF_WANTED_DECLARES: phf::Map<&'static str, &'static str> = phf::phf_map! {
            "function_item" => "f",
            "function_signature_item" => "f",
            "struct_item" => "s",
            "enum_item" => "e",
            "impl_item" => "f",
            "trait_item" => "i",
            "mod_item" => "m",
        };

        /// Rust node kinds treated as imports.
        pub static PHF_WANTED_IMPORTS: phf::Map<&'static str, &'static str> = phf::phf_map! {
            "use_declaration" => "i",
            "mod_item" => "i",
        };
    }

    /// Python grammar node maps.
    pub mod python {
        /// Python node kinds treated as declarations.
        pub static PHF_WANTED_DECLARES: phf::Map<&'static str, &'static str> = phf::phf_map! {
            "function_definition" => "f",
            "class_definition" => "c",
        };

        /// Python node kinds treated as imports.
        pub static PHF_WANTED_IMPORTS: phf::Map<&'static str, &'static str> = phf::phf_map! {
            "import_statement" => "i",
            "import_from_statement" => "i",
        };
    }
}

#[derive(Debug, Clone, Copy)]
/// Definition of a command flag.
pub struct FlagDef {
    /// Logical flag name.
    pub name: &'static str, // the logical name
    /// Flag token or regex value.
    pub value: &'static str, // what are we looking for A
    /// Help text shown to users.
    pub help: &'static str, // help text
    /// Example usage text.
    pub example: &'static str, // help text
}

/// Command flag definitions and grouped flag sets.
pub mod commandflags {

    use super::FlagDef;
    use crate::loader_for_constants;
    use once_cell::sync::Lazy;

    #[derive(Debug, Clone, Copy)]
    /// Effective command flag values after config resolution.
    pub struct EffectiveFlagDef {
        /// Logical flag name.
        pub name: &'static str,
        /// Effective token or regex.
        pub value: &'static str,
        /// Effective help text.
        pub help: &'static str,
        /// Effective example text.
        pub example: &'static str,
    }

    /// Returns effective search flags as plain values.
    pub fn searchflags_effective() -> Vec<EffectiveFlagDef> {
        searchflags()
            .iter()
            .map(|flag| EffectiveFlagDef {
                name: flag.name,
                value: flag.value,
                help: flag.help,
                example: flag.example,
            })
            .collect()
    }

    /// Flag used to split positive and negative name filters.
    pub static NAME_NEGATIVE_SPLITTER: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "name_negative_splitter",
        value: loader_for_constants::resolve("commandflags.NAME_NEGATIVE_SPLITTER.value"),
        help: "split a search into a positive and negative component",
        example: loader_for_constants::resolve("commandflags.NAME_NEGATIVE_SPLITTER.example"),
    });

    /// Flag for case-insensitive matching.
    pub static CASE_INSENSITIVE: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "insensitive",
        value: loader_for_constants::resolve("commandflags.CASE_INSENSITIVE.value"),
        help: "case-insentive search",
        example: loader_for_constants::resolve("commandflags.CASE_INSENSITIVE.example"),
    });

    /// Flag for after-context line count.
    pub static AFTER_CONTEXT: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "after",
        value: loader_for_constants::resolve("commandflags.AFTER_CONTEXT.value"),
        help: "Show NUM lines after each match.",
        example: loader_for_constants::resolve("commandflags.AFTER_CONTEXT.example"),
    });

    /// Flag for before-context line count.
    pub static BEFORE_CONTEXT: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "before",
        value: loader_for_constants::resolve("commandflags.BEFORE_CONTEXT.value"),
        help: "Show NUM lines before each match.",
        example: loader_for_constants::resolve("commandflags.BEFORE_CONTEXT.example"),
    });

    /// Flag for symmetric context line count.
    pub static CONTEXT: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "context",
        value: loader_for_constants::resolve("commandflags.CONTEXT.value"),
        help: "Show NUM lines before and after each match.",
        example: loader_for_constants::resolve("commandflags.CONTEXT.example"),
    });

    /// Flag for assigning a key to matched lines.
    pub static SETKEY: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "set key",
        //closes with a `.` or the string end.
        value: loader_for_constants::resolve("commandflags.SETKEY.value"),
        help: "set key to a value that can be used with wk=<key>. in later commands",
        example: loader_for_constants::resolve("commandflags.SETKEY.example"),
    });

    /// Flag for filtering by a previously set key.
    pub static WHEREKEY: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "where_key",
        value: loader_for_constants::resolve("commandflags.WHEREKEY.value"),
        help: "puts an extra criteria to other commands",
        example: loader_for_constants::resolve("commandflags.WHEREKEY.example"),
    });

    /// Flag for filtering declaration types.
    pub static WHERETYPE: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "where_type",
        value: loader_for_constants::resolve("commandflags.WHERETYPE.value"),
        help: "only show some types of declarations",
        example: loader_for_constants::resolve("commandflags.WHERETYPE.example"),
    });

    /// Flag for filtering by line-number range.
    pub static WHERE_LINENUM: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "where_linenum",
        value: loader_for_constants::resolve("commandflags.WHERE_LINENUM.value"),
        help: "limit line number range",
        example: loader_for_constants::resolve("commandflags.WHERE_LINENUM.example"),
    });

    /// Flag to treat pattern as fixed text.
    pub static FIXED_STRING: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "fixed_string",
        value: loader_for_constants::resolve("commandflags.FIXED_STRING.value"),
        help: "treat search as regular string, not a regex",
        example: loader_for_constants::resolve("commandflags.FIXED_STRING.example"),
    });

    /// Flag requiring user confirmation.
    pub static USER_CONFIRM: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "user_confirm",
        value: loader_for_constants::resolve("commandflags.USER_CONFIRM.value"),
        help: "user confirmation",
        example: loader_for_constants::resolve("commandflags.USER_CONFIRM.example"),
    });

    /// Flag enabling change-all behavior.
    pub static CHANGE_ALL: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "change_all",
        value: loader_for_constants::resolve("commandflags.CHANGE_ALL.value"),
        help: "user confirmation",
        example: loader_for_constants::resolve("commandflags.CHANGE_ALL.example"),
    });

    /// Flag enabling grep shortcode expansion.
    pub static USE_GREP_SHORTCODES: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "use_grep_shortcodes",
        value: loader_for_constants::resolve("commandflags.USE_GREP_SHORTCODES.value"),
        help: "expands shortcodes (`..`→`.*`, `~~s.`→` +`)",
        example: loader_for_constants::resolve("commandflags.USE_GREP_SHORTCODES.example"),
    });

    /// Flag disabling grep shortcode expansion.
    pub static NO_GREP_SHORTCODES: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "no_use_grep_shortcodes",
        value: loader_for_constants::resolve("commandflags.NO_GREP_SHORTCODES.value"),
        help: "does not expand grep shortcodes",
        example: loader_for_constants::resolve("commandflags.NO_GREP_SHORTCODES.example"),
    });

    /// Flag to preserve indentation on inserted text.
    pub static INDENT_KEEP: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "indent_keep",
        value: loader_for_constants::resolve("commandflags.INDENT_KEEP.value"),
        help: "keep matching line's indentation",
        example: loader_for_constants::resolve("commandflags.INDENT_KEEP.example"),
    });

    /// Flag to treat replacement text as raw.
    pub static RAW_STRING: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "raw_string",
        value: loader_for_constants::resolve("commandflags.RAW_STRING.value"),
        help: r"ignore string escapes like `\n`",
        example: loader_for_constants::resolve("commandflags.RAW_STRING.example"),
    });

    static LINESFLAGS_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| vec![*SETKEY, *WHEREKEY]);

    /// Returns flags accepted by the lines command.
    pub fn linesflags() -> &'static [FlagDef] {
        LINESFLAGS_RUNTIME.as_slice()
    }

    static SEARCHFLAGS_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| {
        vec![
            *CASE_INSENSITIVE,
            *AFTER_CONTEXT,
            *BEFORE_CONTEXT,
            *CONTEXT,
            *FIXED_STRING,
            *USE_GREP_SHORTCODES,
            *NO_GREP_SHORTCODES,
            *SHOW_OWNER,
            *SETKEY,
            *WHEREKEY,
            *WHERE_LINENUM,
        ]
    });

    /// Returns flags accepted by search commands.
    pub fn searchflags() -> &'static [FlagDef] {
        SEARCHFLAGS_RUNTIME.as_slice()
    }

    static AND_FLAGS_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| {
        vec![
            *AFTER_CONTEXT,
            *BEFORE_CONTEXT,
            *CONTEXT,
            *CASE_INSENSITIVE,
            *FIXED_STRING,
            *USE_GREP_SHORTCODES,
            *NO_GREP_SHORTCODES,
            *SETKEY,
            *WHEREKEY,
            *WHERE_LINENUM,
        ]
    });

    /// Returns flags accepted by the and command.
    pub fn and_flags() -> &'static [FlagDef] {
        AND_FLAGS_RUNTIME.as_slice()
    }

    static DELETE_FLAGS_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| {
        vec![
            *CASE_INSENSITIVE,
            *FIXED_STRING,
            *USE_GREP_SHORTCODES,
            *NO_GREP_SHORTCODES,
            *WHEREKEY,
            *USER_CONFIRM,
        ]
    });

    /// Returns flags accepted by the delete command.
    pub fn delete_flags() -> &'static [FlagDef] {
        DELETE_FLAGS_RUNTIME.as_slice()
    }

    static CHANGE_FLAGS_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| {
        vec![
            *CHANGE_ALL,
            *CASE_INSENSITIVE,
            *FIXED_STRING,
            *USE_GREP_SHORTCODES,
            *NO_GREP_SHORTCODES,
            *WHEREKEY,
            *USER_CONFIRM,
        ]
    });

    /// Returns flags accepted by the change command.
    pub fn change_flags() -> &'static [FlagDef] {
        CHANGE_FLAGS_RUNTIME.as_slice()
    }

    static PREPEND_APPEND_FLAGS_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| {
        vec![
            *CASE_INSENSITIVE,
            *INDENT_KEEP,
            *FIXED_STRING,
            *USE_GREP_SHORTCODES,
            *NO_GREP_SHORTCODES,
            *WHEREKEY,
            *USER_CONFIRM,
        ]
    });

    /// Returns flags accepted by prepend/append commands.
    pub fn insert_flags() -> &'static [FlagDef] {
        PREPEND_APPEND_FLAGS_RUNTIME.as_slice()
    }

    // things that only go to the Regex engine
    static FOR_REGEX_RUNTIME: Lazy<Vec<FlagDef>> = Lazy::new(|| vec![*CASE_INSENSITIVE]);

    /// Returns flags forwarded only to regex compilation.
    pub fn for_regex() -> &'static [FlagDef] {
        FOR_REGEX_RUNTIME.as_slice()
    }

    /// Flag to include declaration body output.
    pub static SHOW_BODY: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "show_body",
        value: loader_for_constants::resolve("commandflags.SHOW_BODY.value"),
        help: "body of the definition",
        example: loader_for_constants::resolve("commandflags.SHOW_BODY.example"),
    });

    /// Flag to include owner metadata output.
    pub static SHOW_OWNER: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "show_owner",
        value: loader_for_constants::resolve("commandflags.SHOW_OWNER.value"),
        help: "show \"owner\" of the line",
        example: loader_for_constants::resolve("commandflags.SHOW_OWNER.example"),
    });

    static DECLARATION_FLAGS_RUNTIME: Lazy<Vec<FlagDef>> =
        Lazy::new(|| vec![*SHOW_BODY, *SHOW_OWNER, *WHERETYPE]);

    /// Returns flags accepted by declarations commands.
    pub fn declaration_flags() -> &'static [FlagDef] {
        DECLARATION_FLAGS_RUNTIME.as_slice()
    }

    /// Flag used to split name negation filters.
    pub static NAME_NEGATION_SPLITTER: Lazy<FlagDef> = Lazy::new(|| FlagDef {
        name: "name_negation_splitter",
        value: loader_for_constants::resolve("commandflags.NAME_NEGATION_SPLITTER.value"),
        help: "filters out unwanted values on the right",
        example: loader_for_constants::resolve("commandflags.NAME_NEGATION_SPLITTER.example"),
    });
}

/// Grep shortcode definitions and grouped shortcode sets.
pub mod grep_shortcodes {
    use crate::loader_for_constants;
    use once_cell::sync::Lazy;

    #[derive(Debug, Copy, Clone)]
    /// Definition for a grep shortcode macro.
    pub struct GrepShortCodeDef {
        /// Logical shortcode key.
        pub key: &'static str,
        /// Macro token typed by the user.
        pub macrocode: &'static str,
        /// Regex/text expansion for the macro.
        pub expansion: &'static str,
        /// Human-readable macro description.
        pub description: &'static str,
    }

    /// Shortcode for wildcard expansion.
    pub static WILDCARD: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "wildcard",
        macrocode: loader_for_constants::resolve("grep_shortcodes.WILDCARD.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.WILDCARD.expansion"),
        description: "matches any string",
    });

    /// Shortcode for one-or-more spaces.
    pub static SPACES: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "spaces",
        macrocode: loader_for_constants::resolve("grep_shortcodes.SPACES.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.SPACES.expansion"),
        description: "matches one or more space character",
    });

    /// Shortcode for regex OR operator.
    pub static OR: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "or",
        macrocode: loader_for_constants::resolve("grep_shortcodes.OR.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.OR.expansion"),
        description: "standard grep OR/pipe a|b",
    });

    // used in name searches, where characters are [A-Z0-9_-a-z], so other chars can be used
    /// Alternate shortcode for regex OR operator.
    pub static OR_COMMA: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "or_comma",
        macrocode: loader_for_constants::resolve("grep_shortcodes.OR_COMMA.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.OR_COMMA.expansion"),
        description: "standard grep OR/pipe alternative for a|b",
    });

    /// Shortcode for matching quote characters.
    pub static QUOTES: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "quotes",
        macrocode: loader_for_constants::resolve("grep_shortcodes.QUOTES.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.QUOTES.expansion"),
        description: "matches either a single or double quote character",
    });

    /// Shortcode for matching `(`.
    pub static LEFT_PARENTHESIS: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "left_parenthesis",
        macrocode: loader_for_constants::resolve("grep_shortcodes.LEFT_PARENTHESIS.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.LEFT_PARENTHESIS.expansion"),
        description: "matches `(`",
    });

    /// Shortcode for matching `)`.
    pub static RIGHT_PARENTHESIS: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "right_parenthesis",
        macrocode: loader_for_constants::resolve("grep_shortcodes.RIGHT_PARENTHESIS.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.RIGHT_PARENTHESIS.expansion"),
        description: "matches `)`",
    });

    /// Shortcode for matching `<`.
    pub static LESS_THAN: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "less_than",
        macrocode: loader_for_constants::resolve("grep_shortcodes.LESS_THAN.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.LESS_THAN.expansion"),
        description: "matches `<`",
    });

    /// Shortcode for matching field separators.
    pub static FIELD_SEPARATOR: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "field_separator",
        macrocode: loader_for_constants::resolve("grep_shortcodes.FIELD_SEPARATOR.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.FIELD_SEPARATOR.expansion"),
        description: "matches `::`",
    });

    /// Shortcode for matching `:`.
    pub static COLON: Lazy<GrepShortCodeDef> = Lazy::new(|| GrepShortCodeDef {
        key: "colon",
        macrocode: loader_for_constants::resolve("grep_shortcodes.COLON.macrocode"),
        expansion: loader_for_constants::resolve("grep_shortcodes.COLON.expansion"),
        description: "matches `:`",
    });

    /// to replace in REPLACE field
    static REPLACEFIELD_RUNTIME: Lazy<Vec<GrepShortCodeDef>> = Lazy::new(|| vec![*FIELD_SEPARATOR]);

    /// Returns shortcodes used in replacement fields.
    pub fn replacefield() -> &'static [GrepShortCodeDef] {
        REPLACEFIELD_RUNTIME.as_slice()
    }

    static SEARCHFIELD_RUNTIME: Lazy<Vec<GrepShortCodeDef>> = Lazy::new(|| {
        vec![
            *WILDCARD,
            *SPACES,
            *OR,
            *QUOTES,
            *LESS_THAN,
            *LEFT_PARENTHESIS,
            *RIGHT_PARENTHESIS,
            *FIELD_SEPARATOR,
            *COLON,
        ]
    });

    /// Returns shortcodes used in general search fields.
    pub fn searchfield() -> &'static [GrepShortCodeDef] {
        SEARCHFIELD_RUNTIME.as_slice()
    }

    static NAME_SEARCHFIELD_RUNTIME: Lazy<Vec<GrepShortCodeDef>> =
        Lazy::new(|| vec![*WILDCARD, *OR, *OR_COMMA]);

    /// Returns shortcodes used in name-only searches.
    pub fn name_searchfield() -> &'static [GrepShortCodeDef] {
        NAME_SEARCHFIELD_RUNTIME.as_slice()
    }
}

/// Global debug toggle derived from the RUST_LOG environment variable.
pub static DEBUGGING: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
    std::env::var("RUST_LOG")
        .map(|value| value.to_lowercase() == "debug")
        .unwrap_or(false)
});

/// Regex used to split positive and negative name filters.
pub static RE_SPLIT_NEGATIVES_NAMES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(commandflags::NAME_NEGATIVE_SPLITTER.value)
        .expect("NAME_NEGATIVE_SPLITTER regex must be valid")
});
