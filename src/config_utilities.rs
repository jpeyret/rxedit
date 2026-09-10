use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::constants::env_vars;
use log::warn;
use serde::Deserialize;

/// User-visible configuration loaded from the rxedit config file.
#[derive(Debug, Default, Deserialize)]
pub struct UserConfigFile {
    /// Whether to include spacing between declarations.
    pub spacer: Option<bool>,
    /// Whether to include line numbers.
    pub line_number: Option<bool>,
    /// Whether to enable grep shortcode expansion.
    pub grep_shortcodes: Option<bool>,
}

/// Environment overrides for the user configuration.
#[derive(Debug, Default)]
pub struct EnvConfig {
    /// Whether to include spacing between declarations.
    pub spacer: Option<bool>,
    /// Whether to include line numbers.
    pub line_number: Option<bool>,
    /// Whether to enable grep shortcode expansion.
    pub grep_shortcodes: Option<bool>,
}

/**
Returns the canonical path to the user's rxedit config file.
*/
pub fn user_config_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("rxedit")
            .join("config.toml"),
    )
}

/**
Reads the config TOML file contents when the file exists.
*/
pub fn read_config_file_contents() -> Result<Option<String>, String> {
    let Some(path) = user_config_path() else {
        warn!("Config file not found: unable to resolve ~/.config/rxedit/config.toml from HOME");
        return Ok(None);
    };

    if !path.exists() {
        warn!("Config file not found at {}", path.display());
        return Ok(None);
    }

    fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| format!("Cannot read config file '{}': {}", path.display(), e))
}

/**
Returns a display-friendly path for the user config file.
*/
pub fn user_config_path_display() -> String {
    user_config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.config/rxedit/config.toml".to_string())
}

/**
Returns the default TOML content for a new user config file.
*/
pub fn default_user_config_text() -> &'static str {
    "# rxedit user config\n\
# all keys are optional; values are booleans\n\
\n\
spacer = false\n\
line_number = false\n\
grep_shortcodes = false\n\
initial_hide = true\n"
}

/**
Creates the default config file when it is missing.
*/
pub fn write_default_config_if_missing(path: &Path) -> Result<bool, String> {
    if path.exists() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Cannot create config directory '{}': {}",
                parent.display(),
                e
            )
        })?;
    }

    fs::write(path, default_user_config_text())
        .map_err(|e| format!("Cannot write config file '{}': {}", path.display(), e))?;
    Ok(true)
}

/**
Creates the default user config file at the standard rxedit location.
*/
pub fn generate_user_config_file() -> Result<(), String> {
    let Some(path) = user_config_path() else {
        return Err("Cannot resolve user config path from HOME".to_string());
    };

    let created = write_default_config_if_missing(&path)?;
    if created {
        println!("Created config file at {}", path.display());
    } else {
        println!(
            "Config file already exists at {}; not overwriting",
            path.display()
        );
    }

    Ok(())
}

/**
Loads the optional user config file and parses it into the typed config struct.
*/
pub fn read_user_config() -> Result<UserConfigFile, String> {
    let Some(content) = read_config_file_contents()? else {
        return Ok(UserConfigFile::default());
    };

    let Some(path) = user_config_path() else {
        return Ok(UserConfigFile::default());
    };

    toml::from_str::<UserConfigFile>(&content)
        .map_err(|e| format!("Cannot parse config file '{}': {}", path.display(), e))
}

/**
Loads and parses a user config file from an explicit CLI path.
*/
pub fn read_user_config_from_path(path: &Path) -> Result<UserConfigFile, String> {
    if !path.exists() {
        return Err(format!("Config file not found at {}", path.display()));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read config file '{}': {}", path.display(), e))?;

    toml::from_str::<UserConfigFile>(&content)
        .map_err(|e| format!("Cannot parse config file '{}': {}", path.display(), e))
}

fn expand_tilde_path(raw: &str) -> PathBuf {
    if let Some(stripped) = raw.strip_prefix("~/")
        && let Ok(home) = env::var("HOME")
    {
        return PathBuf::from(home).join(stripped);
    }

    if raw == "~" && let Ok(home) = env::var("HOME") {
        return PathBuf::from(home);
    }

    PathBuf::from(raw)
}

fn plugin_directory_from_config_contents(contents: &str) -> Option<PathBuf> {
    let Ok(doc) = contents.parse::<toml::Table>() else {
        return None;
    };

    let plugins = doc.get("plugins")?.as_table()?;
    let path = plugins.get("directory")?.as_str()?;
    Some(expand_tilde_path(path))
}

fn plugin_directory_from_config() -> Option<PathBuf> {
    let Ok(Some(contents)) = read_config_file_contents() else {
        return None;
    };

    plugin_directory_from_config_contents(&contents)
}

fn push_unique(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.iter().any(|existing| existing == &path) {
        candidates.push(path);
    }
}

fn plugin_library_search_paths_from_sources(
    env_plugins_directory: Option<&str>,
    config_plugin_directory: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = env_plugins_directory.map(str::trim)
        && !path.is_empty()
    {
        push_unique(&mut candidates, expand_tilde_path(path));
    }

    if let Some(path) = config_plugin_directory {
        push_unique(&mut candidates, path);
    }

    candidates
}

/// Returns plugin search directories in precedence order.
///
/// Precedence:
/// 1. `rxedit_plugins_directory`
/// 2. `[plugins].directory` from `~/.config/rxedit/config.toml`
pub fn plugin_library_search_paths() -> Vec<PathBuf> {
    plugin_library_search_paths_from_sources(
        env::var(env_vars::PLUGINS_DIRECTORY).ok().as_deref(),
        plugin_directory_from_config(),
    )
}

fn parse_env_bool(var_name: &str) -> Result<Option<bool>, String> {
    let Some(raw_value) = env::var_os(var_name) else {
        return Ok(None);
    };

    let value = raw_value.to_string_lossy().trim().to_ascii_lowercase();
    let parsed = match value.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    };

    parsed
        .map(Some)
        .ok_or_else(|| format!(
            "Invalid boolean value '{}' for env var {}. Use one of: true/false, 1/0, yes/no, on/off",
            value,
            var_name
        ))
}

/**
Loads the config values supplied via environment variables.
*/
pub fn read_env_config() -> Result<EnvConfig, String> {
    Ok(EnvConfig {
        spacer: parse_env_bool(env_vars::SPACER)?,
        line_number: parse_env_bool(env_vars::LINE_NUMBER)?,
        grep_shortcodes: parse_env_bool(env_vars::GREP_SHORTCODES)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        default_user_config_text, plugin_directory_from_config_contents,
        plugin_library_search_paths_from_sources,
        write_default_config_if_missing,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn gen_config_writes_when_missing() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("rxedit-main-test-{}", now));
        let config_path = temp_root.join(".config").join("rxedit").join("config.toml");

        let created = write_default_config_if_missing(&config_path).expect("create config");
        assert!(created);

        let content = fs::read_to_string(&config_path).expect("read created config");
        assert_eq!(content, default_user_config_text());

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn gen_config_does_not_overwrite_existing_file() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("rxedit-main-test-no-overwrite-{}", now));
        let config_path = temp_root.join(".config").join("rxedit").join("config.toml");

        fs::create_dir_all(config_path.parent().expect("config parent")).expect("create dirs");
        fs::write(&config_path, "grep_shortcodes = true\n").expect("seed config");

        let created = write_default_config_if_missing(&config_path).expect("skip overwrite");
        assert!(!created);

        let content = fs::read_to_string(&config_path).expect("read seeded config");
        assert_eq!(content, "grep_shortcodes = true\n");

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn plugin_directory_from_config_contents_reads_plugins_directory() {
        let config = r#"
[plugins]
directory = "~/.config/rxedit/plugins"
"#;

        let path = plugin_directory_from_config_contents(config).expect("plugins.directory");
        assert!(path.ends_with(".config/rxedit/plugins"));
    }

    #[test]
    fn plugin_paths_prioritize_env_over_config() {
        let paths = plugin_library_search_paths_from_sources(
            Some("/tmp/from-new-env"),
            Some(PathBuf::from("/tmp/from-config")),
        );

        assert_eq!(
            paths,
            vec![
                PathBuf::from("/tmp/from-new-env"),
                PathBuf::from("/tmp/from-config"),
            ]
        );
    }
}
