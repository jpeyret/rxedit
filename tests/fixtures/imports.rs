pub mod commands;
use crate::common::CommandQualifier;
pub use commands::all::CAll;
use enum_dispatch::enum_dispatch;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::{collections,path};
use crate::common::{GrepCommandQualifier,LineStatus};
use crate::constants as c;
mod tests {
    use super::preformat;
}
