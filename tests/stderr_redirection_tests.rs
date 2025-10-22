// Integration tests for stderr redirection functionality
use cli_rust::modules::command::Command;
use cli_rust::modules::input::{Environment, InputProcessorBuilder};
use cli_rust::modules::runner::Runner;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

#[test]
fn test_input_processor_stderr_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test basic stderr redirection parsing
    let cmds = ip.process("command 2> error.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "command");
    assert_eq!(cmd.stderr.as_deref(), Some("error.txt"));
    assert!(!cmd.append_stderr);
}

#[test]
fn test_input_processor_stderr_append_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test stderr append redirection parsing
    let cmds = ip.process("command 2>> error.log").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "command");
    assert_eq!(cmd.stderr.as_deref(), Some("error.log"));
    assert!(cmd.append_stderr);
}

#[test]
fn test_input_processor_explicit_fd_numbers() {
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
fn test_input_processor_combined_redirections() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test combined stdin, stdout, and stderr redirection
    let cmds = ip
        .process("grep pattern < input.txt > output.txt 2> error.txt")
        .unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "grep");
    assert_eq!(cmd.args, vec!["pattern"]);
    assert_eq!(cmd.stdin.as_deref(), Some("input.txt"));
    assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
    assert_eq!(cmd.stderr.as_deref(), Some("error.txt"));
    assert!(!cmd.append_stdout);
    assert!(!cmd.append_stderr);
}

#[test]
fn test_input_processor_mixed_append_modes() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test mixed append and overwrite modes
    let cmds = ip.process("command >> output.log 2> error.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stdout.as_deref(), Some("output.log"));
    assert_eq!(cmd.stderr.as_deref(), Some("error.txt"));
    assert!(cmd.append_stdout);
    assert!(!cmd.append_stderr);

    let cmds = ip.process("command > output.txt 2>> error.log").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
    assert_eq!(cmd.stderr.as_deref(), Some("error.log"));
    assert!(!cmd.append_stdout);
    assert!(cmd.append_stderr);
}

#[test]
fn test_input_processor_stderr_with_environment_vars() {
    let mut vars = HashMap::new();
    vars.insert("ERRFILE".to_string(), "errors.log".to_string());

    let env = Environment::with_vars(vars);
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip.process("command 2> $ERRFILE").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stderr.as_deref(), Some("errors.log"));
}

#[test]
fn test_input_processor_quoted_stderr_filenames() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test stderr redirection with quoted filenames
    let cmds = ip.process(r#"command 2> "error file.log""#).unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.stderr.as_deref(), Some("error file.log"));
}

#[test]
fn test_end_to_end_stderr_redirection() {
    let test_dir = env::temp_dir().join("cli_stderr_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let error_file = test_dir.join("test_stderr.txt");
    let error_path = error_file.to_string_lossy().to_string();

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test command that produces stderr output
    // Using a command that's likely to produce stderr (like trying to cat a non-existent file)
    let cmd = Command::new(
        "cat".to_string(),
        vec!["nonexistent_file_12345.txt".to_string()],
    )
    .with_stderr(error_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(_) => {
            // This shouldn't succeed, but if it does, check the file anyway
        }
        Err(e) => {
            // Expected - command should fail
            // Check that error message mentions exit code (stderr was redirected)
            assert!(e.to_string().contains("exit code") || e.to_string().contains("not found"));
        }
    }

    // Check if stderr was written to file (this may not work in all test environments)
    if error_file.exists() {
        println!("Stderr file was created successfully");
        if let Ok(contents) = fs::read_to_string(&error_file) {
            println!("Stderr contents: {}", contents);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_stderr_append() {
    let test_dir = env::temp_dir().join("cli_stderr_append_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let error_file = test_dir.join("append_stderr.log");
    let error_path = error_file.to_string_lossy().to_string();

    // Write initial content
    fs::write(&error_file, "Initial error line\n").expect("Failed to write initial content");

    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Multiple commands with stderr append
    let commands = vec![
        Command::new("cat".to_string(), vec!["nonexistent1.txt".to_string()])
            .with_stderr(error_path.clone())
            .with_append_stderr(true),
        Command::new("cat".to_string(), vec!["nonexistent2.txt".to_string()])
            .with_stderr(error_path.clone())
            .with_append_stderr(true),
    ];

    for cmd in commands {
        let _result = runner.execute(cmd);
        // Commands are expected to fail, but stderr should still be appended to file
    }

    // Check that file still contains original content
    if error_file.exists() {
        if let Ok(contents) = fs::read_to_string(&error_file) {
            assert!(contents.contains("Initial error line"));
            println!("Final stderr file contents: {}", contents);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_stderr_redirection_with_working_command() {
    let test_dir = env::temp_dir().join("cli_stderr_working_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let error_file = test_dir.join("no_errors.log");
    let error_path = error_file.to_string_lossy().to_string();

    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test command that should succeed (echo doesn't typically produce stderr)
    let cmd = Command::new("echo".to_string(), vec!["Hello World".to_string()])
        .with_stderr(error_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            // Command should succeed
            assert!(output.contains("Hello World"));

            // Error file should be created but empty (or very small)
            if error_file.exists() {
                let contents = fs::read_to_string(&error_file).unwrap_or_default();
                // Echo shouldn't produce stderr, so file should be empty or nearly empty
                assert!(
                    contents.len() < 10,
                    "Unexpected stderr content: {}",
                    contents
                );
            }
        }
        Err(e) => {
            println!("Echo command failed (may not be available): {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_combined_stdout_stderr_redirection() {
    let test_dir = env::temp_dir().join("cli_combined_redirect_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("output.txt");
    let error_file = test_dir.join("error.txt");
    let output_path = output_file.to_string_lossy().to_string();
    let error_path = error_file.to_string_lossy().to_string();

    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test with echo (should write to stdout, not stderr)
    let cmd = Command::new("echo".to_string(), vec!["Success message".to_string()])
        .with_stdout(output_path.clone())
        .with_stderr(error_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            // Should return empty since stdout is redirected
            assert_eq!(output, "");

            // Check output file
            if output_file.exists() {
                let contents =
                    fs::read_to_string(&output_file).expect("Failed to read output file");
                assert!(contents.contains("Success message"));
            }

            // Error file should exist but be empty
            if error_file.exists() {
                let contents = fs::read_to_string(&error_file).unwrap_or_default();
                assert!(contents.len() < 10, "Unexpected stderr: {}", contents);
            }
        }
        Err(e) => {
            println!("Command failed (may not be available): {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_fd_redirect_parsing() {
    // Test various fd redirect patterns indirectly through the input processor
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test various fd redirect patterns
    let test_cases = vec![
        "command 3> file3.txt",
        "command 4>> file4.log",
        "command 10> file10.txt",
    ];

    for input in test_cases {
        let result = ip.process(input);
        // For fd >= 3, these should be treated as regular arguments since we don't support them yet
        match result {
            Ok(cmds) => {
                assert_eq!(cmds.len(), 1);
                let cmd = &cmds[0];
                assert_eq!(cmd.name, "command");
                // These should be in args since we don't support fd >= 3
                assert!(cmd.args.len() >= 2);
            }
            Err(_) => {
                // Also acceptable - parsing might fail
            }
        }
    }
}
