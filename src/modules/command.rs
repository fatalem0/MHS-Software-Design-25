/// Common command structure used across the CLI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub append_stdout: bool,
}

impl Command {
    pub fn new<N: Into<String>>(name: N, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            args,
            stdin: None,
            stdout: None,
            append_stdout: false,
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
    }
}