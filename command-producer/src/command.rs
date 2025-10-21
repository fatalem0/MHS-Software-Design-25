#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub Name: String,
    pub Args: Vec<String>,
    pub Stdin: Option<String>,
    pub Stdout: Option<String>,
}
