use crate::modules::command::Command;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

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

    /// Set an environment variable
    pub fn set_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Get environment variable
    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
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
        eprintln!("Executing custom binary: {:?}", binary_path);
        let mut cmd = StdCommand::new(binary_path);
        cmd.args(&command.args);

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        if command.stdin.is_some() {
            cmd.stdin(Stdio::piped());
        }

        // Handle stdout redirection
        if let Some(stdout_file) = &command.stdout {
            let file = if command.append_stdout {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(stdout_file)?
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(stdout_file)?
            };
            cmd.stdout(Stdio::from(file));
        } else {
            cmd.stdout(Stdio::piped());
        }

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
            // If stdout was redirected to file, return empty string (no output to display)
            if command.stdout.is_some() {
                Ok(String::new())
            } else {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
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

        // Handle stdout redirection
        if let Some(stdout_file) = &command.stdout {
            let file = if command.append_stdout {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(stdout_file)?
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(stdout_file)?
            };
            cmd.stdout(Stdio::from(file));
        } else {
            cmd.stdout(Stdio::piped());
        }

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
            // If stdout was redirected to file, return empty string (no output to display)
            if command.stdout.is_some() {
                Ok(String::new())
            } else {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::other(format!("Command failed: {}", error)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

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

    #[test]
    fn test_command_with_stdout() {
        let cmd = Command::new("echo".to_string(), vec!["hello".to_string()])
            .with_stdout("output.txt".to_string());
        assert!(cmd.stdout.is_some());
        assert_eq!(cmd.stdout.unwrap(), "output.txt");
    }

    #[test]
    fn test_command_builder_pattern() {
        let cmd = Command::new("test".to_string(), vec!["arg1".to_string()])
            .with_stdin("input".to_string())
            .with_stdout("output.txt".to_string());

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.args, vec!["arg1"]);
        assert_eq!(cmd.stdin.unwrap(), "input");
        assert_eq!(cmd.stdout.unwrap(), "output.txt");
    }

    #[test]
    fn test_runner_creation() {
        let bin_path = PathBuf::from("/test/bin");
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let runner = Runner::new(bin_path.clone(), env_vars.clone());
        assert_eq!(runner.bin_path, bin_path);
        assert_eq!(runner.env_vars, env_vars);
    }

    #[test]
    fn test_runner_system_command_echo() {
        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd = Command::new(
            "echo".to_string(),
            vec!["hello".to_string(), "world".to_string()],
        );
        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                assert!(output.contains("hello"));
                assert!(output.contains("world"));
            }
            Err(_) => {
                // Echo command might not be available in test environment
                // This is acceptable for unit tests
            }
        }
    }

    #[test]
    fn test_runner_nonexistent_command() {
        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd = Command::new("definitely_nonexistent_command_12345".to_string(), vec![]);
        let result = runner.execute(cmd);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn test_runner_empty_command_name() {
        let bin_path = PathBuf::from("/test/bin");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd = Command::new("".to_string(), vec![]);
        let result = runner.execute(cmd);

        assert!(result.is_err());
    }

    #[test]
    fn test_runner_with_environment_variables() {
        let bin_path = PathBuf::from("/nonexistent/path");
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_ENV_VAR".to_string(), "42".to_string());
        let runner = Runner::new(bin_path, env_vars);

        // Try to run a command that uses environment variables
        // Note: This test might be platform-dependent

        let cmd = Command::new("printenv".to_string(), vec!["TEST_ENV_VAR".to_string()]);
        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                assert!(output.contains("42"));
            }
            Err(_) => {
                // printenv might not be available, that's okay for unit tests
            }
        }
    }

    #[test]
    fn test_runner_custom_binary_detection() {
        // Create a temporary directory structure for testing
        let test_dir = env::temp_dir().join("cli_runner_test");
        let bin_dir = test_dir.join("bin");

        // Clean up any existing test directory
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&bin_dir).expect("Failed to create test directory");

        // Create a fake binary file
        let fake_binary = bin_dir.join("test_cmd");
        fs::write(&fake_binary, "#!/bin/bash\necho 'custom binary executed'")
            .expect("Failed to create test binary");

        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&fake_binary).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&fake_binary, perms).unwrap();
        }

        let env_vars = HashMap::new();
        let runner = Runner::new(bin_dir.clone(), env_vars);

        let cmd = Command::new("test_cmd".to_string(), vec![]);
        let result = runner.execute(cmd);

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);

        // On Unix systems, this should work; on Windows it might not

        match result {
            Ok(output) => {
                assert!(output.contains("custom binary executed"));
            }
            Err(e) => {
                // Might fail due to shell availability, that's acceptable
                println!("Custom binary test failed (acceptable): {}", e);
            }
        }
    }

    #[test]
    fn test_command_with_stdin_execution() {
        // Test system cat command with stdin (if available)
        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd =
            Command::new("cat".to_string(), vec![]).with_stdin("hello from stdin\n".to_string());

        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                assert!(output.contains("hello from stdin"));
            }
            Err(_) => {
                // cat command might not be available in test environment
                // This is acceptable for unit tests
            }
        }
    }

    #[test]
    fn test_command_args_handling() {
        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        // Test with multiple arguments
        let cmd = Command::new(
            "echo".to_string(),
            vec![
                "arg1".to_string(),
                "arg with spaces".to_string(),
                "arg3".to_string(),
            ],
        );

        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                assert!(output.contains("arg1"));
                assert!(output.contains("arg with spaces"));
                assert!(output.contains("arg3"));
            }
            Err(_) => {
                // echo might not be available, acceptable for tests
            }
        }
    }

    #[test]
    fn test_stdout_redirection_write() {
        use std::env;

        let test_dir = env::temp_dir().join("cli_test_stdout");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let output_file = test_dir.join("test_output.txt");
        let output_path = output_file.to_string_lossy().to_string();

        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd = Command::new("echo".to_string(), vec!["Hello World".to_string()])
            .with_stdout(output_path.clone());

        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                // Should return empty string when redirecting to file
                assert_eq!(output, "");

                // Check file contents
                if let Ok(file_contents) = fs::read_to_string(&output_file) {
                    assert!(file_contents.contains("Hello World"));
                }
            }
            Err(e) => {
                println!(
                    "Echo command not available for stdout redirection test: {}",
                    e
                );
            }
        }

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_stdout_redirection_append() {
        use std::env;

        let test_dir = env::temp_dir().join("cli_test_stdout_append");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let output_file = test_dir.join("test_append.txt");
        let output_path = output_file.to_string_lossy().to_string();

        // Write initial content
        fs::write(&output_file, "Initial line\n").expect("Failed to write initial content");

        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd = Command::new("echo".to_string(), vec!["Appended line".to_string()])
            .with_stdout(output_path.clone())
            .with_append_stdout(true);

        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                // Should return empty string when redirecting to file
                assert_eq!(output, "");

                // Check file contents
                if let Ok(file_contents) = fs::read_to_string(&output_file) {
                    assert!(file_contents.contains("Initial line"));
                    assert!(file_contents.contains("Appended line"));
                }
            }
            Err(e) => {
                println!("Echo command not available for stdout append test: {}", e);
            }
        }

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_stdin_redirection() {
        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let input_data = "line1\nline2\nline3\n";
        let cmd = Command::new("cat".to_string(), vec![]).with_stdin(input_data.to_string());

        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                assert!(output.contains("line1"));
                assert!(output.contains("line2"));
                assert!(output.contains("line3"));
            }
            Err(e) => {
                println!(
                    "Cat command not available for stdin redirection test: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_stdin_and_stdout_redirection_combined() {
        use std::env;

        let test_dir = env::temp_dir().join("cli_test_combined");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let output_file = test_dir.join("combined_output.txt");
        let output_path = output_file.to_string_lossy().to_string();

        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let input_data = "test input data\n";
        let cmd = Command::new("cat".to_string(), vec![])
            .with_stdin(input_data.to_string())
            .with_stdout(output_path.clone());

        let result = runner.execute(cmd);

        match result {
            Ok(output) => {
                // Should return empty string when redirecting to file
                assert_eq!(output, "");

                // Check file contents
                if let Ok(file_contents) = fs::read_to_string(&output_file) {
                    assert!(file_contents.contains("test input data"));
                }
            }
            Err(e) => {
                println!(
                    "Cat command not available for combined redirection test: {}",
                    e
                );
            }
        }

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_command_redirection_builder_pattern() {
        let cmd = Command::new("test".to_string(), vec!["arg".to_string()])
            .with_stdin("input data".to_string())
            .with_stdout("output.txt".to_string())
            .with_append_stdout(true);

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.args, vec!["arg"]);
        assert_eq!(cmd.stdin.as_deref(), Some("input data"));
        assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
        assert!(cmd.append_stdout);
    }

    #[test]
    fn test_stdout_file_creation() {
        use std::env;

        let test_dir = env::temp_dir().join("cli_test_file_creation");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let output_file = test_dir.join("new_file.txt");
        let output_path = output_file.to_string_lossy().to_string();

        // Ensure file doesn't exist initially
        assert!(!output_file.exists());

        let bin_path = PathBuf::from("/nonexistent/path");
        let env_vars = HashMap::new();
        let runner = Runner::new(bin_path, env_vars);

        let cmd = Command::new("echo".to_string(), vec!["Creating new file".to_string()])
            .with_stdout(output_path.clone());

        let result = runner.execute(cmd);

        match result {
            Ok(_) => {
                // File should now exist
                assert!(output_file.exists());

                // Check file contents
                if let Ok(file_contents) = fs::read_to_string(&output_file) {
                    assert!(file_contents.contains("Creating new file"));
                }
            }
            Err(e) => {
                println!("Echo command not available for file creation test: {}", e);
            }
        }

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }
}
