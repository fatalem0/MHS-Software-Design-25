use crate::modules::command::Command;
use crate::modules::init::Init;
use crate::modules::input::{Environment, InputProcessor, InputProcessorBuilder};
use crate::modules::runner::Runner;

use std::collections::HashMap;
use std::fs;
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

        let env: Environment = Environment::with_vars(_env_vars.clone());
        let input_processor = InputProcessorBuilder::new(env).build();

        Repl {
            bin_path,
            _env_vars,
            runner,
            input_processor,
        }
    }

    pub fn run(&mut self) {
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

                    // Check if it's a variable assignment (NAME=VALUE)
                    if self.is_variable_assignment(input) {
                        self.handle_variable_assignment(input);
                        continue;
                    }

                    // Process as command
                    match self.input_processor.process(input) {
                        Ok(parsed_cmds) => {
                            // If any parsed command expands to a builtin like `exit` or `help`,
                            // handle it here (after expansion). This allows constructs like
                            // $x$y to expand to `exit` and be treated as the builtin.
                            let mut should_break = false;
                            for pc in parsed_cmds {
                                // Handle builtins after expansion
                                if pc.name == "exit" && pc.args.is_empty() {
                                    println!("Goodbye!");
                                    should_break = true;
                                    break;
                                }
                                if pc.name == "help" && pc.args.is_empty() {
                                    self.show_help();
                                    continue;
                                }

                                // Convert parsed command to runner::Command with redirection support
                                let mut cmd = Command::new(pc.name.clone(), pc.args.clone());

                                // Add redirection information
                                if let Some(stdin_file) = pc.stdin {
                                    // Read the file content for stdin redirection
                                    match fs::read_to_string(&stdin_file) {
                                        Ok(content) => {
                                            cmd = cmd.with_stdin(content);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error reading stdin file '{}': {}",
                                                stdin_file, e
                                            );
                                            continue;
                                        }
                                    }
                                }
                                if let Some(stdout) = pc.stdout {
                                    cmd = cmd
                                        .with_stdout(stdout)
                                        .with_append_stdout(pc.append_stdout);
                                }
                                if let Some(stderr) = pc.stderr {
                                    cmd = cmd
                                        .with_stderr(stderr)
                                        .with_append_stderr(pc.append_stderr);
                                }

                                self.execute_command(cmd);
                            }
                            if should_break {
                                break;
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

    fn is_variable_assignment(&self, input: &str) -> bool {
        // Simple check for pattern NAME=VALUE where NAME is a valid identifier
        if let Some(eq_pos) = input.find('=') {
            let name_part = &input[..eq_pos];
            // Check if name part is a valid identifier (starts with letter/underscore, contains alphanumeric/underscore)
            if !name_part.is_empty()
                && name_part.chars().all(|c| c.is_alphanumeric() || c == '_')
                && (name_part.chars().next().unwrap().is_alphabetic() || name_part.starts_with('_'))
            {
                return true;
            }
        }
        false
    }

    fn handle_variable_assignment(&mut self, input: &str) {
        if let Some(eq_pos) = input.find('=') {
            let name = &input[..eq_pos];
            let value = &input[eq_pos + 1..];

            // Update environment in input processor
            if let Some(env) = self.input_processor.get_environment_mut() {
                env.set(name.to_string(), value.to_string());
                println!("Set {}={}", name, value);
            } else {
                eprintln!("Failed to set environment variable");
            }

            // Also update runner's environment
            self.runner.set_env_var(name.to_string(), value.to_string());
        }
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("Built-in commands:");
        println!("  echo [args...]     - Print arguments to stdout");
        println!("  cat [files...]     - Display file contents or read from stdin");
        println!("  wc [files...]      - Count lines, words, and bytes in files or stdin");
        println!("  pwd               - Print current working directory");
        println!("  help              - Show this help message");
        println!("  exit              - Exit the shell");
        println!();
        println!("Shell features:");
        println!("  NAME=VALUE         - Set environment variable");
        println!("  $VAR or ${{VAR}}     - Variable expansion");
        println!("  cmd < file         - Redirect stdin from file");
        println!("  cmd > file         - Redirect stdout to file (overwrite)");
        println!("  cmd >> file        - Redirect stdout to file (append)");
        println!("  cmd 2> file        - Redirect stderr to file (overwrite)");
        println!("  cmd 2>> file       - Redirect stderr to file (append)");
        // println!("  cmd1 | cmd2        - Pipe output between commands");
        println!("  [command]          - Execute any system command or fallback to built-in");
    }
}

impl Default for Repl {
    fn default() -> Self {
        let init = Init::new();
        Self::new(&init)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_repl_creation() {
        let init = Init::new();
        let repl = Repl::new(&init);

        assert_eq!(repl.bin_path, init.bin_path);
        assert_eq!(repl._env_vars, init.env_vars().clone());
    }

    #[test]
    fn test_repl_default() {
        let _repl = Repl::default();
        // Should not panic
    }

    #[test]
    fn test_variable_assignment_detection() {
        let repl = Repl::default();

        // Valid assignments
        assert!(repl.is_variable_assignment("VAR=value"));
        assert!(repl.is_variable_assignment("PATH=/usr/bin"));
        assert!(repl.is_variable_assignment("_PRIVATE=secret"));
        assert!(repl.is_variable_assignment("VAR123=test"));

        // Invalid assignments
        assert!(!repl.is_variable_assignment("echo hello"));
        assert!(!repl.is_variable_assignment("=value"));
        assert!(!repl.is_variable_assignment("123VAR=value"));
        assert!(!repl.is_variable_assignment("VAR-NAME=value"));
        assert!(!repl.is_variable_assignment(""));
    }

    #[test]
    fn test_stdin_file_reading() {
        let test_dir = env::temp_dir().join("cli_repl_test_stdin");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let input_file = test_dir.join("test_input.txt");
        let test_content = "Test file content\nLine 2\nLine 3\n";
        fs::write(&input_file, test_content).expect("Failed to write test file");

        // Test file reading functionality indirectly by checking if fs::read_to_string works
        // (The actual REPL stdin file reading is tested in integration tests)
        let content = fs::read_to_string(&input_file).expect("Failed to read test file");
        assert_eq!(content, test_content);
        assert!(content.contains("Test file content"));
        assert!(content.contains("Line 2"));
        assert!(content.contains("Line 3"));

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_stdin_file_reading_error_handling() {
        // Test reading non-existent file
        let result = fs::read_to_string("/nonexistent/file.txt");
        assert!(result.is_err());

        // The error handling in REPL will print an error message and continue
        // This behavior is tested in integration tests
    }

    #[test]
    fn test_command_creation_with_redirection() {
        // Test the command creation logic that the REPL uses
        let name = "cat".to_string();
        let args = vec![];
        let mut cmd = Command::new(name.clone(), args.clone());

        // Simulate stdin redirection
        let stdin_content = "file content".to_string();
        cmd = cmd.with_stdin(stdin_content.clone());

        // Simulate stdout redirection
        let stdout_file = "output.txt".to_string();
        cmd = cmd
            .with_stdout(stdout_file.clone())
            .with_append_stdout(false);

        assert_eq!(cmd.name, name);
        assert_eq!(cmd.args, args);
        assert_eq!(cmd.stdin, Some(stdin_content));
        assert_eq!(cmd.stdout, Some(stdout_file));
        assert!(!cmd.append_stdout);
    }

    #[test]
    fn test_command_creation_with_append_redirection() {
        let name = "echo".to_string();
        let args = vec!["test".to_string()];
        let mut cmd = Command::new(name.clone(), args.clone());

        // Simulate stdout append redirection
        let stdout_file = "output.txt".to_string();
        cmd = cmd
            .with_stdout(stdout_file.clone())
            .with_append_stdout(true);

        assert_eq!(cmd.name, name);
        assert_eq!(cmd.args, args);
        assert!(cmd.stdin.is_none());
        assert_eq!(cmd.stdout, Some(stdout_file));
        assert!(cmd.append_stdout);
    }

    #[test]
    fn test_environment_variable_handling() {
        let init = Init::new();
        let mut repl = Repl::new(&init);

        // Test setting environment variable through runner
        repl.runner
            .set_env_var("TEST_VAR".to_string(), "test_value".to_string());

        // Test getting environment variable
        let value = repl.runner.get_env_var("TEST_VAR");
        assert_eq!(value, Some(&"test_value".to_string()));

        // Test non-existent variable
        let no_value = repl.runner.get_env_var("NONEXISTENT_VAR");
        assert_eq!(no_value, None);
    }
}
