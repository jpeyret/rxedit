use crate::commands::prelude::*;

#[derive(Debug)]
/// Flips visibility for all lines.
pub struct CInvert;

/// Returns static definition metadata for this command.
pub fn get_constant_definition() -> CommandDefinition {
    CommandDefinition {
        name: command_prefix::INVERT,
        mutates_line_text: false,
        ..Default::default()
    }
}

impl CommandActions for CInvert {
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {
        lines
            .into_iter()
            .map(|mut line_status| {
                line_status.visible = !line_status.visible;
                line_status
            })
            .collect()
    }

    fn get_constant_definition(&self) -> CommandDefinition {
        get_constant_definition()
    }
}
