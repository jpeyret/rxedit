//! tracks whether an extension can be tested, based on availability of dynamic libraries

pub use crate::support::languages::{plugin_library_filename, resolve_plugin_library_path};

// do we have the plugin?
fn plugin_available(ext: &str) -> bool {
    match resolve_plugin_library_path(ext) {
        Some(_path) => true,
        None => false,
    }
}

/// used to disable plugin testing on the CI if the environment is not configured for it
pub fn zsh_plugin_tests_enabled() -> bool {
    plugin_available("zsh")
}

/// used to disable plugin testing on the CI if the environment is not configured for it
pub fn js_plugin_tests_enabled() -> bool {
    plugin_available("js")
}
