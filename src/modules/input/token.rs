#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenMode {
    Raw,   // '...'
    Weak,  // "..."
    Full,  // unquoted
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub value: String,
    pub mode: TokenMode,
}
impl Token {
    pub fn new<S: Into<String>>(value: S, mode: TokenMode) -> Self {
        Self { value: value.into(), mode }
    }
}
