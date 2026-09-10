use crate::commands::prelude::*;
use crate::common::request_explain;

#[derive(Debug)]
/// Emits debug output for currently stored telemetry events.
pub struct CExplain;

impl CommandActions for CExplain {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        // most of the actual work is done in main::run
        request_explain();
        lines
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        CommandDefinition {
            name: command_prefix::EXPLAIN,
            ..Default::default()
        }
    }
}
