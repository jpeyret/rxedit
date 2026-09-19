use crate::Command;
use crate::base::CheckCondition;
use crate::commands::prelude::*;
use crate::common;
use crate::common::TelemetryEvent;
use crate::common::{GrepCommandQualifier, LineStatus};
use crate::constants as c;

#[derive(Debug)]
/// Shows a range of lines
pub struct CLines {
    /// Line-range condition used by this command.
    pub condition: common::CheckLineRange,
    /// instructions for the command
    pub qualifier: GrepCommandQualifier,
}

pub(crate) fn from_arg(arg: &str, payload: &str, flags: &str) -> (TelemetryEvent, Command) {
    let condition = common::CheckLineRange::new(payload.to_string());
    let allowed_flags = c::commandflags::linesflags();
    let (qualifier, _unconsumed) = GrepCommandQualifier::build_with_flags(flags, allowed_flags);

    let s_qualifier = format!("{}", qualifier);
    let s_searcher = format!("{}", condition);

    let telemetry_event = TelemetryEvent::GenericCommandNotification {
        name: "lines".to_string(),
        arg: arg.to_string(),
        qualifier: s_qualifier,
        searcher: s_searcher,
        unrecognized: _unconsumed.clone(),
    };

    let command = Command::CLines(CLines {
        condition,
        qualifier,
    });
    (telemetry_event, command)
}

fn f_calculate_visibility(visible: bool, hit: bool) -> bool {
    visible || hit
}

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::LINES,
        mutates_line_text: false,
        file_arg_policy: FileArgPolicy::No,
        flags: commandflags::linesflags(),
    }
}

impl CommandActions for CLines {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        let total_lines = lines.len();
        lines
            .into_iter()
            .map(|mut line_status| {
                let hit = self.condition.check_with_total_lines(&line_status, total_lines)
                    && self
                        .qualifier
                        .conditions_checker
                        .check_with_total_lines(&line_status, total_lines);
                if hit && let Some(key) = &self.qualifier.set_key {
                    line_status.meta.key = key.clone();
                }
                line_status.visible = f_calculate_visibility(line_status.visible, hit);
                line_status
            })
            .collect()
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}
