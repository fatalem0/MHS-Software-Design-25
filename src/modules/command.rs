/// Common command structure used across the CLI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub append_stdout: bool,
    pub stderr: Option<String>,
    pub append_stderr: bool,
}

impl Command {
    pub fn new<N: Into<String>>(name: N, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            args,
            stdin: None,
            stdout: None,
            append_stdout: false,
            stderr: None,
            append_stderr: false,
        }
    }

    pub fn with_stdin<S: Into<String>>(mut self, stdin: S) -> Self {
        self.stdin = Some(stdin.into());
        self
    }

    pub fn with_stdout<S: Into<String>>(mut self, stdout: S) -> Self {
        self.stdout = Some(stdout.into());
        self
    }

    pub fn with_append_stdout(mut self, append: bool) -> Self {
        self.append_stdout = append;
        self
    }

    pub fn with_stderr<S: Into<String>>(mut self, stderr: S) -> Self {
        self.stderr = Some(stderr.into());
        self
    }

    pub fn with_append_stderr(mut self, append: bool) -> Self {
        self.append_stderr = append;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("echo", vec!["hello".to_string()]);
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello"]);
        assert!(cmd.stdin.is_none());
        assert!(cmd.stdout.is_none());
        assert!(!cmd.append_stdout);
    }

    #[test]
    fn test_command_builder_pattern() {
        let cmd = Command::new("test", vec!["arg1".to_string()])
            .with_stdin("input")
            .with_stdout("output.txt")
            .with_append_stdout(true);

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.args, vec!["arg1"]);
        assert_eq!(cmd.stdin.unwrap(), "input");
        assert_eq!(cmd.stdout.unwrap(), "output.txt");
        assert!(cmd.append_stdout);
        assert!(cmd.stderr.is_none());
        assert!(!cmd.append_stderr);
    }

    #[test]
    fn test_command_with_stderr() {
        let cmd = Command::new("test", vec!["arg".to_string()]).with_stderr("errors.log");

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.stderr.unwrap(), "errors.log");
        assert!(!cmd.append_stderr);
    }

    #[test]
    fn test_command_with_stderr_append() {
        let cmd = Command::new("test", vec![])
            .with_stderr("errors.log")
            .with_append_stderr(true);

        assert_eq!(cmd.stderr.unwrap(), "errors.log");
        assert!(cmd.append_stderr);
    }

    #[test]
    fn test_command_full_redirection() {
        let cmd = Command::new("test", vec!["arg".to_string()])
            .with_stdin("input.txt")
            .with_stdout("output.txt")
            .with_stderr("error.txt")
            .with_append_stdout(false)
            .with_append_stderr(true);

        assert_eq!(cmd.stdin.unwrap(), "input.txt");
        assert_eq!(cmd.stdout.unwrap(), "output.txt");
        assert_eq!(cmd.stderr.unwrap(), "error.txt");
        assert!(!cmd.append_stdout);
        assert!(cmd.append_stderr);
    }
}
