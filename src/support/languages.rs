use crate::config_utilities;
use std::path::PathBuf;

/// dll? so? dylib? extension?
pub fn plugin_library_filename(language_id: &str) -> String {
    let ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(windows) {
        "dll"
    } else {
        "so"
    };

    format!("librxedit_language_{language_id}.{ext}")
}

/// find the dynamic libraty for the language
pub fn resolve_plugin_library_path(language_id: &str) -> Option<PathBuf> {
    let candidate_name = plugin_library_filename(language_id);

    for directory in config_utilities::plugin_library_search_paths() {
        let candidate = directory.join(&candidate_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(test)]
mod tests {

    use super::plugin_library_filename;

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
}
