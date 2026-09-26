use crate::base::FileArgPolicy;
use crate::commands::prelude::*;
use crate::common;
use crate::common::LineStatus;

#[derive(Debug)]
/// Shows a range of lines
pub struct CMacro;

// note:  lib::expand_macros does the actual work with macros, so there is no need to create one.
// or to do anything with it.

impl CommandActions for CMacro {
    //this is a do-nothing pass-thru at time of execution
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        lines
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        CommandDefinition {
            name: command_prefix::MACROS,
            mutates_line_text: false,
            file_arg_policy: FileArgPolicy::Yes,
            ..Default::default()
        }
    }
}
