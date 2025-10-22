// Core integration tests for CLI functionality
// Tests: input parsing, tokenization, variable expansion, redirection parsing, error handling
use pretty_assertions::assert_eq;
use std::collections::HashMap;

use cli_rust::modules::input::errors::CliError;
use cli_rust::modules::input::{Environment, InputProcessorBuilder};

#[test]
fn test_environment_operations() {
    let mut env = Environment::new();
    env.set("USER", "alice");
    assert_eq!(env.get("USER"), Some("alice"));
    env.remove("USER");
    assert_eq!(env.get("USER"), None);
}

#[test]
fn test_tokenization_quotes_and_expansion() {
    let mut vars = HashMap::new();
    vars.insert("NAME".to_string(), "Bob".to_string());
    vars.insert("X".to_string(), "1".to_string());
    let env = Environment::with_vars(vars);
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip
        .process(r#"echo "hi $NAME" '$NAME' world \\ \$X > out.txt"#)
        .unwrap();
    assert_eq!(cmds.len(), 1);
    let c0 = &cmds[0];
    assert_eq!(c0.name, "echo");
    assert_eq!(
        c0.args,
        vec![
            "hi Bob".to_string(),
            "$NAME".to_string(),
            "world".to_string(),
            "\\".to_string(),
            "$X".to_string()
        ]
    );
    assert_eq!(c0.stdout.as_deref(), Some("out.txt"));
    assert!(!c0.append_stdout);
}

#[test]
fn test_pipeline_parsing() {
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
fn test_stdin_stdout_redirection_parsing() {
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
fn test_stderr_redirection_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Basic stderr redirection
    let cmds = ip.process("command 2> error.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "command");
    assert_eq!(cmd.stderr.as_deref(), Some("error.txt"));
    assert!(!cmd.append_stderr);

    // stderr append
    let cmds = ip.process("command 2>> error.log").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert!(cmd.append_stderr);

    // Combined redirections
    let cmds = ip
        .process("grep pattern < input.txt > output.txt 2> error.txt")
        .unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stdin.as_deref(), Some("input.txt"));
    assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
    assert_eq!(cmd.stderr.as_deref(), Some("error.txt"));
}

#[test]
fn test_explicit_file_descriptors() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test explicit fd numbers
    let test_cases = vec![
        ("command 0< input.txt", Some("input.txt"), None, None),
        ("command 1> output.txt", None, Some("output.txt"), None),
        ("command 2> error.txt", None, None, Some("error.txt")),
    ];

    for (input, expected_stdin, expected_stdout, expected_stderr) in test_cases {
        let cmds = ip.process(input).unwrap();
        assert_eq!(cmds.len(), 1);
        let cmd = &cmds[0];
        assert_eq!(cmd.name, "command");
        assert_eq!(cmd.stdin.as_deref(), expected_stdin);
        assert_eq!(cmd.stdout.as_deref(), expected_stdout);
        assert_eq!(cmd.stderr.as_deref(), expected_stderr);
    }
}

#[test]
fn test_variable_expansion_in_redirections() {
    let mut vars = HashMap::new();
    vars.insert("OUTFILE".to_string(), "result.txt".to_string());
    vars.insert("ERRFILE".to_string(), "errors.log".to_string());

    let env = Environment::with_vars(vars);
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip
        .process("cat < input.txt > $OUTFILE 2> $ERRFILE")
        .unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stdout.as_deref(), Some("result.txt"));
    assert_eq!(cmd.stderr.as_deref(), Some("errors.log"));
}

#[test]
fn test_quoted_filenames_in_redirections() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip
        .process(r#"command < "input file.txt" > "output file.txt" 2> "error file.log""#)
        .unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stdin.as_deref(), Some("input file.txt"));
    assert_eq!(cmd.stdout.as_deref(), Some("output file.txt"));
    assert_eq!(cmd.stderr.as_deref(), Some("error file.log"));
}

#[test]
fn test_parse_errors() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Unclosed quote should error
    let err = ip.process("echo \"oops").unwrap_err();
    assert_eq!(err, CliError::Quote("unclosed quote".into()));
}

#[test]
fn test_adjacent_variable_expansion() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), "ex".to_string());
    vars.insert("y".to_string(), "it".to_string());
    vars.insert("A".to_string(), "1".to_string());
    vars.insert("B".to_string(), "2".to_string());

    let env = Environment::with_vars(vars);
    let ip = InputProcessorBuilder::new(env).build();

    // Test $x$y -> exit
    let cmds = ip.process("$x$y").unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].name, "exit");

    // Test pre$A$Bp -> pre12p
    let cmds = ip.process("echo pre$A$Bp").unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].args, vec!["pre12p"]);
}

#[test]
fn test_mixed_append_modes() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test mixed append and overwrite modes
    let cmds = ip.process("command >> output.log 2> error.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert!(cmd.append_stdout);
    assert!(!cmd.append_stderr);

    let cmds = ip.process("command > output.txt 2>> error.log").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert!(!cmd.append_stdout);
    assert!(cmd.append_stderr);
}
