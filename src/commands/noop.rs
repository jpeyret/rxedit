use crate::commands::prelude::*;

#[derive(Debug)]
/// No-op command that leaves lines unchanged.
pub struct CNoop {
    /// Original command text.
    pub text: String,
    /// User-facing explanation for the no-op.
    pub message: String,
}

impl CommandActions for CNoop {
    fn get_constant_definition(&self) -> CommandDefinition {
        CommandDefinition {
            name: "noop",
            ..Default::default()
        }
    }
}
