use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

/// Represents a command to be executed by the Runner
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
}

impl Command {
    pub fn new(name: String, args: Vec<String>) -> Self {
        Self {
            name,
            args,
            stdin: None,
            stdout: None,
        }
    }

    pub fn with_stdin(mut self, stdin: String) -> Self {
        self.stdin = Some(stdin);
        self
    }

    pub fn with_stdout(mut self, stdout: String) -> Self {
        self.stdout = Some(stdout);
        self
    }
}

/// Runner executes commands, using custom implementations when available
/// or falling back to system executables
pub struct Runner {
    bin_path: PathBuf,
    env_vars: HashMap<String, String>,
}

impl Runner {
    pub fn new(bin_path: PathBuf, env_vars: HashMap<String, String>) -> Self {
        Self { bin_path, env_vars }
    }

    /// Execute a command using custom implementation or system executable
    pub fn execute(&self, command: Command) -> io::Result<String> {
        let custom_binary_path = self.bin_path.join(&command.name);

        if custom_binary_path.exists() {
            self.execute_custom_binary(&command, &custom_binary_path)
        } else {
            self.execute_system_command(&command)
        }
    }

    /// Execute a custom binary from our bin directory
    fn execute_custom_binary(
        &self,
        command: &Command,
        binary_path: &PathBuf,
    ) -> io::Result<String> {
        let mut cmd = StdCommand::new(binary_path);
        cmd.args(&command.args);

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        if command.stdin.is_some() {
            cmd.stdin(Stdio::piped());
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Write to stdin if provided
        if let Some(stdin_data) = &command.stdin {
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                stdin.write_all(stdin_data.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::other(format!("Command failed: {}", error)))
        }
    }

    /// Execute a system command (fallback when no custom implementation exists)
    fn execute_system_command(&self, command: &Command) -> io::Result<String> {
        let mut cmd = StdCommand::new(&command.name);
        cmd.args(&command.args);

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        if command.stdin.is_some() {
            cmd.stdin(Stdio::piped());
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Command '{}' not found: {}", command.name, e),
            )
        })?;

        // Write to stdin if provided
        if let Some(stdin_data) = &command.stdin {
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                stdin.write_all(stdin_data.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::other(format!("Command failed: {}", error)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("echo".to_string(), vec!["hello".to_string()]);
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello"]);
        assert!(cmd.stdin.is_none());
        assert!(cmd.stdout.is_none());
    }

    #[test]
    fn test_command_with_stdin() {
        let cmd = Command::new("cat".to_string(), vec![]).with_stdin("input data".to_string());
        assert!(cmd.stdin.is_some());
        assert_eq!(cmd.stdin.unwrap(), "input data");
    }
}
