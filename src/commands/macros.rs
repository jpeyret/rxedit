use crate::Command;
use crate::commands::prelude::*;
use crate::common;
use crate::common::LineStatus;
use crate::common::TelemetryEvent;

use crate::constants::DEBUGGING;

#[derive(Debug)]
/// Shows a range of lines
pub struct CMacro;

pub(crate) fn from_arg(arg: &str, payload: &str) -> (TelemetryEvent, Command) {
    if *DEBUGGING {
        dbg!(&arg, &payload);
    }

    let telemetry_event = TelemetryEvent::GenericCommandNotification {
        name: "macro".to_string(),
        arg: arg.to_string(),
        qualifier: "".to_string(),
        searcher: payload.to_string(),
        unrecognized: "".to_string(),
    };

    let command = Command::CMacro(CMacro);
    (telemetry_event, command)
}

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
            ..Default::default()
        }
    }
}
