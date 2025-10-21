use std::collections::HashMap;

use crate::command::Command;
use crate::command_name::CommandName;
use crate::command_processor::CommandProcessorMap;

pub struct CommandProducer {
    processors: HashMap<CommandName, Box<dyn crate::command_processor::CommandProcessor>>,
}

impl CommandProducer {
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
        }
    }

    pub fn ProduceCommands(&mut self, input: Vec<String>) -> Vec<Command> {
        input.into_iter().map(|cmd_str| {
            let parts: Vec<String> = cmd_str.split_whitespace().map(|s| s.to_string()).collect();
            if parts.is_empty() {
                Command {
                    Name: String::new(),
                    Args: vec![],
                    Stdin: None,
                    Stdout: None,
                }
            } else {
                Command {
                    Name: parts[0].clone(),
                    Args: parts[1..].to_vec(),
                    Stdin: None,
                    Stdout: None,
                }
            }
        }).collect()
    }

    pub fn RegisterCmdProcessors(&mut self, processors: CommandProcessorMap) {
        self.processors = processors;
    }
}
