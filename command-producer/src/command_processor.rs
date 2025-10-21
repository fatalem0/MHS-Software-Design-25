use std::collections::HashMap;

use crate::command::Command;
use crate::command_name::CommandName;

pub trait CommandProcessor {
    fn process(&self, command: &Command) -> Result<(), Box<dyn std::error::Error>>;
}

pub type CommandProcessorMap = HashMap<CommandName, Box<dyn CommandProcessor>>;
