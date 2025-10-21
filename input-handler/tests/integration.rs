// Integration tests
use std::collections::HashMap;
use pretty_assertions::assert_eq;

use cli_core::{Environment, InputProcessorBuilder};
use cli_core::errors::CliError;

#[test]
fn env_get_set() {
    let mut env = Environment::new();
    env.set("USER", "alice");
    assert_eq!(env.get("USER"), Some("alice"));
    env.remove("USER");
    assert_eq!(env.get("USER"), None);
}

#[test]
fn tokenization_and_quotes_and_expand() {
    let mut vars = HashMap::new();
    vars.insert("NAME".to_string(), "Bob".to_string());
    vars.insert("X".to_string(), "1".to_string());
    let env = Environment::with_vars(vars);
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip.process(r#"echo "hi $NAME" '$NAME' world \\ \$X > out.txt"#).unwrap();
    assert_eq!(cmds.len(), 1);
    let c0 = &cmds[0];
    assert_eq!(c0.name, "echo");
    assert_eq!(c0.args, vec!["hi Bob".to_string(), "$NAME".to_string(), "world".to_string(), "\\".to_string(), "$X".to_string()]);
    assert_eq!(c0.stdout.as_deref(), Some("out.txt"));
    assert!(!c0.append_stdout);
}

#[test]
fn pipeline_three_commands() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip.process(r#"cat "a|b" | grep a | wc -l"#).unwrap();
    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0].name, "cat");
    assert_eq!(cmds[1].name, "grep");
    assert_eq!(cmds[2].name, "wc");
    assert_eq!(cmds[2].args, vec!["-l".to_string()]);
}

#[test]
fn redirections_append_and_input() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();
    let cmds = ip.process(r#"grep foo < in.txt >> out.log"#).unwrap();
    assert_eq!(cmds.len(), 1);
    let c = &cmds[0];
    assert_eq!(c.name, "grep");
    assert_eq!(c.stdin.as_deref(), Some("in.txt"));
    assert_eq!(c.stdout.as_deref(), Some("out.log"));
    assert!(c.append_stdout);
}

#[test]
fn errors_unclosed_quote() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();
    let err = ip.process("echo \"oops").unwrap_err();
    assert_eq!(err, CliError::Quote("unclosed quote".into()));
}

#[test]
fn print_parsed_commands() {
    use cli_core::{Environment, InputProcessorBuilder};

    // Можно менять эту строку для проверки разных сценариев
    let line = r#"echo "Hello $USER" | grep Hello > result.txt"#;

    let env = Environment::capture_current();
    let ip = InputProcessorBuilder::new(env).build();
    let cmds = ip.process(line).expect("failed to parse");

    println!("\nParsed commands for input:\n  {line}\n");
    for (i, cmd) in cmds.iter().enumerate() {
        println!("Command #{}:", i + 1);
        println!("  name   : {}", cmd.name);
        println!("  args   : {:?}", cmd.args);
        if let Some(stdin) = &cmd.stdin {
            println!("  stdin  : {}", stdin);
        }
        if let Some(stdout) = &cmd.stdout {
            println!("  stdout : {} (append: {})", stdout, cmd.append_stdout);
        }
        println!();
    }
}

// Вспомогательный принтер, чтобы не дублировать код в тестах
fn dump_commands(title: &str, line: &str, env: cli_core::Environment) {
    use cli_core::InputProcessorBuilder;
    let ip = InputProcessorBuilder::new(env).build();
    let cmds = ip.process(line).expect("failed to parse");
    println!("\n=== {title} ===\nINPUT: {line}\n");
    for (i, cmd) in cmds.iter().enumerate() {
        println!("Command #{}:", i + 1);
        println!("  name   : {}", cmd.name);
        println!("  args   : {:?}", cmd.args);
        if let Some(stdin) = &cmd.stdin {
            println!("  stdin  : {}", stdin);
        }
        if let Some(stdout) = &cmd.stdout {
            println!("  stdout : {} (append: {})", stdout, cmd.append_stdout);
        }
        println!();
    }
}

#[test]
fn print_redirect_out() {
    use std::collections::HashMap;
    use cli_core::Environment;

    // Простая команда с выводом в файл
    let mut vars = HashMap::new();
    vars.insert("WORD".into(), "hello".into());
    let env = Environment::with_vars(vars);

    let line = r#"echo "$WORD world" > out.txt"#;
    dump_commands("redirect stdout >", line, env);
}

#[test]
fn print_redirect_in_and_append_with_pipe() {
    use cli_core::Environment;

    // Вход < и апенд >> после пайпа
    let env = Environment::new();
    let line = r#"grep foo < in.txt | sort >> out.log"#;
    dump_commands("stdin < and append >> with pipe", line, env);
}

#[test]
fn print_multiple_commands_and_redirects() {
    use cli_core::Environment;

    // Несколько команд в пайплайне, редирект только у последней
    let env = Environment::new();
    let line = r#"cat data.csv | cut -d, -f2 | uniq -c > counts.txt"#;
    dump_commands("pipeline with final stdout redirect", line, env);
}

#[test]
fn print_literal_dollar_no_expand() {
    use std::collections::HashMap;
    use cli_core::Environment;

    // Показываем, что \$ не разворачивается
    let mut vars = HashMap::new();
    vars.insert("X".into(), "42".into());
    let env = Environment::with_vars(vars);

    let line = r#"echo "\$X" $X > out.txt"#;
    dump_commands(r#"escaped \$ vs expanded $X"#, line, env);
}

#[test]
fn print_overwrite_vs_append() {
    use cli_core::Environment;

    // Демонстрация различия > (перезапись) и >> (аппенд)
    // (наш парсер хранит только флаг append и имя файла)
    let env = Environment::new();

    dump_commands("overwrite >", r#"echo one > file.txt"#, env.clone());
    dump_commands("append >>",   r#"echo two >> file.txt"#, env);
}


