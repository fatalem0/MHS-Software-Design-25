use crate::modules::input::{
    command::Command,
    environment::Environment,
    errors::{CliError, Result},
    expander::Expander,
    quote_handler::QuoteHandler,
    tokenizer::Tokenizer,
};

pub struct InputProcessorBuilder {
    env: Environment,
}
impl InputProcessorBuilder {
    pub fn new(env: Environment) -> Self {
        Self { env }
    }
    pub fn build(self) -> InputProcessor {
        InputProcessor {
            env: self.env,
            expander: Expander::default(),
        }
    }
}

#[derive(Clone)]
pub struct InputProcessor {
    env: Environment,
    expander: Expander,
}

impl InputProcessor {
    pub fn process(&self, line: &str) -> Result<Vec<Command>> {
        // 1) Токенизируем всю строку (учитывая кавычки/экраны)
        let raw = Tokenizer::tokenize(line)?;

        // 2) Делим список токенов по некавычённому токену "|"
        let parts = split_on_pipes_tokens(&raw);

        // 3) Обрабатываем каждую часть отдельной командой
        let mut cmds = Vec::with_capacity(parts.len());
        for raw_part in parts {
            let tokens = QuoteHandler::handle(&raw_part)?;
            let pieces = self.expander.expand_tokens(&self.env, tokens)?;
            cmds.push(self.produce_command(pieces)?);
        }
        Ok(cmds)
    }

    fn produce_command(&self, mut pieces: Vec<String>) -> Result<Command> {
        if pieces.is_empty() {
            return Err(CliError::EmptyCommand);
        }
        let name = pieces.remove(0);
        let mut args = Vec::<String>::new();
        let mut stdin = None;
        let mut stdout = None;
        let mut append_stdout = false;

        let mut it = pieces.into_iter().peekable();
        while let Some(p) = it.next() {
            match p.as_str() {
                "<" => stdin = it.next(),
                ">" => {
                    append_stdout = false;
                    stdout = it.next();
                }
                ">>" => {
                    append_stdout = true;
                    stdout = it.next();
                }
                _ => args.push(p),
            }
        }
        let mut cmd = Command::new(name, args);
        cmd.stdin = stdin;
        cmd.stdout = stdout;
        cmd.append_stdout = append_stdout;
        Ok(cmd)
    }
}

fn split_pipes(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut quote: Option<char> = None;
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if quote.is_none() && (c == '\'' || c == '"') {
            quote = Some(c);
        } else if quote == Some(c) {
            quote = None;
        } else if quote.is_none() && c == '|' {
            out.push(line[start..].trim());
            start = i + 1;
        }
        i += 1;
    }
    out.push(&line[start..].trim());
    out
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
