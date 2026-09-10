//! parsable languages are stored under this directory.
//! !!!TODO!!! future roadmap is to replace this via the use of .so modules to facilitate
//! the addition of more supported languages.
/// Python language integration.
pub mod python;
/// Rust language integration.
pub mod rust;

pub(crate) fn phf_keys(map: &'static phf::Map<&'static str, &'static str>) -> Vec<&'static str> {
    map.keys().copied().collect()
}
