use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CliError {
    #[error("tokenization error: {0}")]
    Tokenization(String),
    #[error("quote error: {0}")]
    Quote(String),
    #[error("expansion error: {0}")]
    Expansion(String),
    #[error("empty command")]
    EmptyCommand,
}
pub type Result<T> = std::result::Result<T, CliError>;
