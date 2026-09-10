//! Responsible for loading user-facing constants from their
//! TOML definition files.  At compile time, but also with
//! runtime override capability against ~/.config/rxedit/config.toml

#![allow(dead_code)]

use log::warn;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use toml::Value;

/// Attribute map for a single config entry.
pub type Attributes = HashMap<String, String>;
/// Entry map within a config section.
pub type Entries = HashMap<String, Attributes>;
/// Top-level section map for loaded config.
pub type Sections = HashMap<String, Entries>;

const CONFIGURED_SECTIONS: &[&str] = &["commandflags", "grep_shortcodes"];
const HELP_SECTION: &str = "help";
const HELP_TOPICS_KEY: &str = "topics_help";

fn user_config_path_display() -> String {
    user_config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.config/rxedit/config.toml".to_string())
}

fn fail_config(message: &str) -> ! {
    panic!(
        "\nConfiguration error:\n  {}\n\nExpected sources:\n  - src/defaultconfig.toml\n  - {}\n",
        message,
        user_config_path_display()
    )
}

fn user_config_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("rxedit")
            .join("config.toml"),
    )
}

fn to_nested_string_maps(doc: &Value, source_name: &str) -> Sections {
    let strict = source_name == "src/defaultconfig.toml";
    let root = doc
        .as_table()
        .unwrap_or_else(|| fail_config(&format!("{} must be a TOML table", source_name)));

    let mut sections: Sections = HashMap::new();

    for (section_name, section_value) in root {
        if !CONFIGURED_SECTIONS.iter().any(|name| *name == section_name) {
            continue;
        }

        let Some(section_table) = section_value.as_table() else {
            if strict {
                fail_config(&format!(
                    "{}: section '{}' must be a TOML table",
                    source_name, section_name
                ));
            }
            continue;
        };

        let mut entries: Entries = HashMap::new();
        for (entry_name, entry_value) in section_table {
            let entry_table = entry_value.as_table().unwrap_or_else(|| {
                fail_config(&format!(
                    "{}: '{}.{}' must be a TOML table",
                    source_name, section_name, entry_name
                ))
            });

            let mut attrs: Attributes = HashMap::new();
            for (attr_name, attr_value) in entry_table {
                let attr_str = attr_value.as_str().unwrap_or_else(|| {
                    fail_config(&format!(
                        "{}: '{}.{}.{}' must be a string",
                        source_name, section_name, entry_name, attr_name
                    ))
                });
                attrs.insert(attr_name.clone(), attr_str.to_string());
            }
            entries.insert(entry_name.clone(), attrs);
        }
        sections.insert(section_name.clone(), entries);
    }

    sections
}

fn merge_nested_maps(mut base: Sections, overlay: Sections) -> Sections {
    for (section_name, overlay_entries) in overlay {
        let base_entries = base.entry(section_name).or_default();
        for (entry_name, overlay_attrs) in overlay_entries {
            let base_attrs = base_entries.entry(entry_name).or_default();
            for (attr_name, attr_value) in overlay_attrs {
                base_attrs.insert(attr_name, attr_value);
            }
        }
    }
    base
}

fn parse_help_topics_array(value: &Value, source_name: &str) -> Vec<String> {
    let Some(arr) = value.as_array() else {
        fail_config(&format!(
            "{}: '{}.{}' must be an array of strings",
            source_name, HELP_SECTION, HELP_TOPICS_KEY
        ));
    };

    arr.iter()
        .map(|item| {
            item.as_str().unwrap_or_else(|| {
                fail_config(&format!(
                    "{}: '{}.{}' must contain only strings",
                    source_name, HELP_SECTION, HELP_TOPICS_KEY
                ))
            })
        })
        .map(ToString::to_string)
        .collect()
}

fn parse_help_topics(doc: &Value, source_name: &str) -> Option<Vec<String>> {
    let root = doc
        .as_table()
        .unwrap_or_else(|| fail_config(&format!("{} must be a TOML table", source_name)));

    let help = root.get(HELP_SECTION)?;
    let help_table = help.as_table().unwrap_or_else(|| {
        fail_config(&format!(
            "{}: section '{}' must be a TOML table",
            source_name, HELP_SECTION
        ))
    });

    let topics = help_table.get(HELP_TOPICS_KEY)?;
    Some(parse_help_topics_array(topics, source_name))
}

static EFFECTIVE: Lazy<Sections> = Lazy::new(|| {
    let default_doc: Value = toml::from_str(include_str!("defaultconfig.toml"))
        .expect("src/defaultconfig.toml must be valid TOML");
    let default_map = to_nested_string_maps(&default_doc, "src/defaultconfig.toml");

    let mut effective = default_map;
    if let Some(path) = user_config_path()
        && path.exists()
    {
        let content = fs::read_to_string(&path).unwrap_or_else(|err| {
            fail_config(&format!(
                "Cannot read user config '{}': {}",
                path.display(),
                err
            ))
        });
        let user_doc: Value = toml::from_str(&content).unwrap_or_else(|err| {
            fail_config(&format!(
                "Cannot parse user config '{}': {}",
                path.display(),
                err
            ))
        });
        let user_map = to_nested_string_maps(&user_doc, &format!("{}", path.display()));
        effective = merge_nested_maps(effective, user_map);
    } else {
        warn!("Config file not found at {}", user_config_path_display());
    }

    effective
});

static EFFECTIVE_HELP_TOPICS: Lazy<Vec<String>> = Lazy::new(|| {
    let default_doc: Value = toml::from_str(include_str!("defaultconfig.toml"))
        .expect("src/defaultconfig.toml must be valid TOML");
    let mut effective =
        parse_help_topics(&default_doc, "src/defaultconfig.toml").unwrap_or_else(|| {
            fail_config(&format!(
                "Missing required config key '{}.{}' in src/defaultconfig.toml",
                HELP_SECTION, HELP_TOPICS_KEY
            ))
        });

    if let Some(path) = user_config_path()
        && path.exists()
    {
        let content = fs::read_to_string(&path).unwrap_or_else(|err| {
            fail_config(&format!(
                "Cannot read user config '{}': {}",
                path.display(),
                err
            ))
        });
        let user_doc: Value = toml::from_str(&content).unwrap_or_else(|err| {
            fail_config(&format!(
                "Cannot parse user config '{}': {}",
                path.display(),
                err
            ))
        });
        if let Some(user_topics) = parse_help_topics(&user_doc, &format!("{}", path.display())) {
            effective = user_topics;
        }
    } else {
        warn!("Config file not found at {}", user_config_path_display());
    }

    effective
});

/// Returns an optional config value from section, entry, and attribute.
pub fn get_optional(section: &str, entry: &str, attr: &str) -> Option<&'static str> {
    EFFECTIVE
        .get(section)
        .and_then(|entries| entries.get(entry))
        .and_then(|attrs| attrs.get(attr))
        .map(String::as_str)
}

/// Returns a required config value or fails with a config error.
pub fn get_required(section: &str, entry: &str, attr: &str) -> &'static str {
    get_optional(section, entry, attr).unwrap_or_else(|| {
        fail_config(&format!(
            "Missing required config key '{}.{}.{}' in merged config",
            section, entry, attr
        ))
    })
}

/// Resolves a config value from a dotted path.
pub fn resolve(path: &str) -> &'static str {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() != 3 {
        fail_config(&format!(
            "Invalid config path '{}': expected format 'section.entry.attribute' (got {} parts)",
            path,
            parts.len()
        ));
    }
    get_required(parts[0], parts[1], parts[2])
}

/// Returns the effective help topic list.
pub fn help_topics() -> &'static [String] {
    EFFECTIVE_HELP_TOPICS.as_slice()
}
