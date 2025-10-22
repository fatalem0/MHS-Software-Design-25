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
}
