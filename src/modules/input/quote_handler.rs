use crate::modules::input::errors::{CliError, Result};
use crate::modules::input::token::{Token, TokenMode};

pub struct QuoteHandler;

impl QuoteHandler {
    pub fn handle(raw: &[String]) -> Result<Vec<Token>> {
        let mut out = Vec::with_capacity(raw.len());
        for piece in raw {
            let mut mode = TokenMode::Full;
            let s = piece.as_str();
            let cleaned = if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
                mode = TokenMode::Raw;
                s[1..s.len() - 1].to_string()
            } else if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                mode = TokenMode::Weak;
                unescape_double_quoted(&s[1..s.len() - 1])
            } else {
                unescape_unquoted(s)
            };
            out.push(Token::new(cleaned, mode));
        }
        Ok(out)
    }
}

fn unescape_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(n) = it.next() {
                if n == '$' {
                    // сохраняем backslash перед $, чтобы предотвратить расширение
                    out.push('\\');
                    out.push('$');
                } else {
                    // для остальных — снимаем экранирование
                    out.push(n);
                }
            }
        } else {
            out.push(c)
        }
    }
    out
}
fn unescape_unquoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(n) = it.next() {
                if n == '$' {
                    // сохраняем \ перед $, чтобы дальше Expander распознал \$
                    out.push('\\');
                    out.push('$');
                } else {
                    out.push(n);
                }
            }
        } else {
            out.push(c)
        }
    }
    out
}
