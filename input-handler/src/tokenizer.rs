use crate::errors::{CliError, Result};

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize(line: &str) -> Result<Vec<String>> {
        let mut out = Vec::<String>::new();
        let mut buf = String::new();
        let mut chars = line.chars().peekable();
        let mut quote: Option<char> = None;

        while let Some(c) = chars.next() {
            match (quote, c) {
                (None, ' ' | '\t') => {
                    if !buf.is_empty() { out.push(std::mem::take(&mut buf)); }
                }
                (None, '\'' | '"') => { quote = Some(c); buf.push(c); }
                (Some(q), ch) if ch == q => { buf.push(ch); quote = None; }
                (_, '\\') => {
                    if let Some(n) = chars.next() {
                        buf.push('\\'); buf.push(n);
                    } else {
                        buf.push('\\');
                    }
                }
                _ => buf.push(c),
            }
        }
        if quote.is_some() {
            return Err(CliError::Quote("unclosed quote".into()));
        }
        if !buf.is_empty() { out.push(buf); }
        Ok(out)
    }
}
