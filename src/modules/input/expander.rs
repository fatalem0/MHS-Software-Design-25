use regex::Regex;

use crate::modules::environment::Environment;
use crate::modules::input::errors::Result;
use crate::modules::input::token::{Token, TokenMode};

#[derive(Clone)]
pub struct Expander {
    re_braced: Regex,
}

impl Default for Expander {
    fn default() -> Self {
        Self {
            re_braced: Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap(),
        }
    }
}

impl Expander {
    pub fn expand_tokens(&self, env: &Environment, tokens: Vec<Token>) -> Result<Vec<String>> {
        tokens
            .into_iter()
            .map(|t| self.expand_token(env, t))
            .collect()
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

        // 2) Подстановки ${VAR} (braced variables first - they are unambiguous)
        let step = self
            .re_braced
            .replace_all(&protected, |caps: &regex::Captures| {
                env.get(&caps[1]).unwrap_or("").to_string()
            });

        // 3) Подстановки $VAR (simple variables) using longest match strategy
        let mut result = String::new();
        let chars = step.chars().collect::<Vec<_>>();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() {
                let next_char = chars[i + 1];
                if next_char.is_ascii_alphabetic() || next_char == '_' {
                    // Start of a variable name - try to find the longest match
                    let start_pos = i + 1;
                    let mut end_pos = start_pos;

                    // Collect all valid identifier characters
                    while end_pos < chars.len() {
                        let ch = chars[end_pos];
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            end_pos += 1;
                        } else {
                            break;
                        }
                    }

                    // Try longest match first, then shorter matches
                    let mut matched = false;
                    for try_end in (start_pos..=end_pos).rev() {
                        let var_name: String = chars[start_pos..try_end].iter().collect();
                        if !var_name.is_empty() && env.get(&var_name).is_some() {
                            // Found a match!
                            result.push_str(env.get(&var_name).unwrap());
                            i = try_end; // Continue from after the variable name
                            matched = true;
                            break;
                        }
                    }

                    if !matched {
                        // No variable found, keep the $ and continue
                        result.push(chars[i]);
                        i += 1;
                    }
                } else {
                    // Not a valid variable start, keep the $
                    result.push(chars[i]);
                    i += 1;
                }
            } else {
                // Regular character
                result.push(chars[i]);
                i += 1;
            }
        }

        // 4) Вернём защищённые '$'
        result
            .chars()
            .map(|c| if c == S { '$' } else { c })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::environment::Environment;
    use crate::modules::input::token::{Token, TokenMode};

    #[test]
    fn test_adjacent_variable_expansion() {
        let mut env = Environment::new();
        env.set("x", "ex");
        env.set("y", "it");

        let exp = Expander::default();
        let tokens = vec![Token::new("$x$y", TokenMode::Full)];
        let res = exp.expand_tokens(&env, tokens).expect("expand failed");
        assert_eq!(res, vec!["exit".to_string()]);
    }

    #[test]
    fn test_mixed_adjacent_and_literal() {
        let mut env = Environment::new();
        env.set("A", "1");
        env.set("B", "2");

        let exp = Expander::default();
        // Test the case: pre$A$Bp should expand to pre12p, not pre1 (where $Bp is treated as undefined variable)
        let tokens = vec![Token::new("pre$A$Bp", TokenMode::Full)];
        let res = exp.expand_tokens(&env, tokens).expect("expand failed");
        assert_eq!(res, vec!["pre12p".to_string()]);

        // Also test braced variables to ensure deterministic behavior
        let tokens = vec![Token::new("pre${A}${B}post", TokenMode::Full)];
        let res = exp.expand_tokens(&env, tokens).expect("expand failed");
        assert_eq!(res, vec!["pre12post".to_string()]);
    }

    #[test]
    fn test_variable_with_trailing_text() {
        let mut env = Environment::new();
        env.set("VAR", "value");

        let exp = Expander::default();
        let tokens = vec![Token::new("$VARtext", TokenMode::Full)];
        let res = exp.expand_tokens(&env, tokens).expect("expand failed");
        assert_eq!(res, vec!["valuetext".to_string()]);
    }
}
