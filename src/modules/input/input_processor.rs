use crate::modules::input::{
    command::Command,
    errors::{CliError, Result},
    expander::Expander,
    quote_handler::QuoteHandler,
    tokenizer::Tokenizer,
    Environment,
};

/// CommandProducer is responsible for converting parsed tokens into Command objects
/// with proper redirection handling (stdin, stdout, stderr).
pub struct CommandProducer;

impl CommandProducer {
    /// Produces a Command object from tokenized command pieces, handling redirection operators.
    ///
    /// This method parses redirection operators like `>`, `>>`, `2>`, `<` etc. and builds
    /// a Command with appropriate stdin, stdout, and stderr configurations.
    pub fn produce_command(mut pieces: Vec<String>) -> Result<Command> {
        if pieces.is_empty() {
            return Err(CliError::EmptyCommand);
        }
        let name = pieces.remove(0);
        let mut args = Vec::<String>::new();
        let mut stdin = None;
        let mut stdout = None;
        let mut append_stdout = false;
        let mut stderr = None;
        let mut append_stderr = false;

        let mut it = pieces.into_iter().peekable();
        while let Some(p) = it.next() {
            match p.as_str() {
                "<" | "0<" => stdin = it.next(),
                ">" | "1>" => {
                    append_stdout = false;
                    stdout = it.next();
                }
                ">>" | "1>>" => {
                    append_stdout = true;
                    stdout = it.next();
                }
                "2>" => {
                    append_stderr = false;
                    stderr = it.next();
                }
                "2>>" => {
                    append_stderr = true;
                    stderr = it.next();
                }
                _ => {
                    // Check for patterns like "3>", "4>>", etc.
                    if let Some(fd_redirect) = parse_fd_redirect(&p) {
                        let target_file = it.next();
                        match fd_redirect {
                            (0, false) => stdin = target_file, // "0>"  (unusual but possible)
                            (1, false) => {
                                append_stdout = false;
                                stdout = target_file;
                            } // "1>"
                            (1, true) => {
                                append_stdout = true;
                                stdout = target_file;
                            } // "1>>"
                            (2, false) => {
                                append_stderr = false;
                                stderr = target_file;
                            } // "2>"
                            (2, true) => {
                                append_stderr = true;
                                stderr = target_file;
                            } // "2>>"
                            _ => {
                                // For fd >= 3, we could extend Command struct to support them
                                // For now, ignore or add to args
                                args.push(p);
                                if let Some(file) = target_file {
                                    args.push(file);
                                }
                            }
                        }
                    } else {
                        args.push(p);
                    }
                }
            }
        }
        let mut cmd = Command::new(name, args);
        cmd.stdin = stdin;
        cmd.stdout = stdout;
        cmd.append_stdout = append_stdout;
        cmd.stderr = stderr;
        cmd.append_stderr = append_stderr;
        Ok(cmd)
    }
}

pub struct InputProcessorBuilder {}
impl InputProcessorBuilder {
    pub fn new() -> Self {
        Self {}
    }
    pub fn build(self) -> InputProcessor {
        InputProcessor {
            expander: Expander::default(),
        }
    }
}

impl Default for InputProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// InputProcessor acts as a facade that coordinates tokenization, variable expansion,
/// and command production to convert input strings into executable commands.
#[derive(Clone)]
pub struct InputProcessor {
    expander: Expander,
}

impl InputProcessor {
    pub fn process(&self, line: &str, env_vars: &Environment) -> Result<Vec<Command>> {
        // 1) Токенизируем всю строку (учитывая кавычки/экраны)
        let raw = Tokenizer::tokenize(line)?;

        // 2) Делим список токенов по некавычённому токену "|"
        let parts = split_on_pipes_tokens(&raw);

        // 3) Обрабатываем каждую часть отдельной командой
        let mut cmds = Vec::with_capacity(parts.len());
        for raw_part in parts {
            let tokens = QuoteHandler::handle(&raw_part)?;
            let pieces = self.expander.expand_tokens(env_vars, tokens)?;
            cmds.push(CommandProducer::produce_command(pieces)?);
        }
        Ok(cmds)
    }
}

/// Parse file descriptor redirection patterns like "2>", "3>>", "1>", etc.
/// Returns Some((fd_number, is_append)) if the string matches a pattern, None otherwise.
fn parse_fd_redirect(s: &str) -> Option<(u32, bool)> {
    if s.ends_with(">>") {
        if let Some(fd_part) = s.strip_suffix(">>") {
            if let Ok(fd) = fd_part.parse::<u32>() {
                return Some((fd, true)); // append mode
            }
        }
    } else if let Some(fd_part) = s.strip_suffix('>') {
        // Pattern like "2>" or "1>"
        if let Ok(fd) = fd_part.parse::<u32>() {
            return Some((fd, false)); // overwrite mode
        }
    }
    None
}

/// Делит уже токенизированную строку на команды по токену "|".
/// Токен "|" будет отдельным элементом только если он вне кавычек.
fn split_on_pipes_tokens(raw: &[String]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for t in raw {
        if t == "|" {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(t.clone());
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
