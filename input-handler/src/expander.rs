use regex::Regex;

use crate::environment::Environment;
use crate::errors::Result;
use crate::token::{Token, TokenMode};

#[derive(Clone)]
pub struct Expander {
    re_simple: Regex,
    re_braced: Regex,
}

impl Default for Expander {
    fn default() -> Self {
        Self {
            re_simple: Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
            re_braced: Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap(),
        }
    }
}

impl Expander {
    pub fn expand_tokens(&self, env: &Environment, tokens: Vec<Token>) -> Result<Vec<String>> {
        tokens.into_iter().map(|t| self.expand_token(env, t)).collect()
    }

    fn expand_token(&self, env: &Environment, token: Token) -> Result<String> {
        Ok(match token.mode {
            TokenMode::Raw => token.value,
            TokenMode::Weak | TokenMode::Full => self.expand_vars(env, &token.value),
        })
    }

    fn expand_vars(&self, env: &Environment, s: &str) -> String {
        // 1) Защитим экранированные `$`: `\$` -> sentinel (удаляем backslash).
        const S: char = '\u{0001}';
        let mut protected = String::with_capacity(s.len());
        let mut it = s.chars().peekable();
        while let Some(c) = it.next() {
            if c == '\\' {
                if let Some('$') = it.peek().copied() {
                    let _ = it.next(); // съесть '$'
                    protected.push(S);
                    continue;
                } else {
                    protected.push('\\');
                    continue;
                }
            }
            protected.push(c);
        }

        // 2) Подстановки $VAR и ${VAR}
        let step = self.re_braced.replace_all(&protected, |caps: &regex::Captures| {
            env.get(&caps[1]).unwrap_or("").to_string()
        });
        let step = self.re_simple.replace_all(&step, |caps: &regex::Captures| {
            env.get(&caps[1]).unwrap_or("").to_string()
        }).into_owned();

        // 3) Вернём защищённые '$'
        step.chars().map(|c| if c == S { '$' } else { c }).collect()
    }
}
