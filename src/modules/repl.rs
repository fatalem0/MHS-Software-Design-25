use crate::modules::init::Init;
use crate::modules::runner::{Command, Runner};
use crate::modules::input::{Environment, InputProcessor, InputProcessorBuilder};

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct Repl {
    bin_path: PathBuf,
    _env_vars: HashMap<String, String>,
    runner: Runner,
    input_processor: InputProcessor,
}

impl Repl {
    pub fn new(init: &Init) -> Self {
        let bin_path = init.bin_path.clone();
        let _env_vars = init.env_vars().clone();
        let runner = Runner::new(bin_path.clone(), _env_vars.clone());


        let env = Environment::with_vars(_env_vars.clone());
        let input_processor = InputProcessorBuilder::new(env).build();

        Repl {
            bin_path,
            _env_vars,
            runner,
            input_processor,
        }
    }

    pub fn run(&self) {
        println!("CLI Shell started with bin path: {:?}", self.bin_path);
        println!("Type 'exit' to quit or 'help' for available commands.");

        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    if input.is_empty() {
                        continue;
                    }

                    if input == "exit" {
                        println!("Goodbye!");
                        break;
                    }

                    if input == "help" {
                        self.show_help();
                        continue;
                    }

                    // For now, create a simple command from the input
                    // In the future, this will be replaced with proper parsing and tokenization
                    match self.input_processor.process(input) {
                        Ok(parsed_cmds) => {
                            // Минимальная интеграция: выполняем команды последовательно.
                            // (Если Runner позже будет уметь пайплайны — здесь можно заменить на execute_pipeline)
                            for pc in parsed_cmds {
                                // Конвертация parsed -> runner::Command (пока игнорируем редиректы;
                                // если нужно — учтите pc.stdin / pc.stdout / pc.append_stdout в Runner)
                                let cmd = Command::new(pc.name.clone(), pc.args.clone());
                                self.execute_command(cmd);
                            }
                        }
                        Err(e) => eprintln!("parse error: {e}"),
                    }
                }
                Err(error) => {
                    eprintln!("Error reading input: {}", error);
                    break;
                }
            }
        }
    }

    fn execute_command(&self, command: Command) {
        match self.runner.execute(command) {
            Ok(output) => {
                if !output.trim().is_empty() {
                    print!("{}", output);
                }
            }
            Err(error) => {
                eprintln!("Error executing command: {}", error);
            }
        }
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  echo [args...]  - Print arguments to stdout");
        println!("  help           - Show this help message");
        println!("  exit           - Exit the shell");
        println!("  [command]      - Execute any system command or custom implementation");
    }
}

impl Default for Repl {
    fn default() -> Self {
        let init = Init::new();
        Self::new(&init)
    }
}
