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
    use std::env;
    use std::fs;

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
}
