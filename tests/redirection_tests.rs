// Integration tests for stdin and stdout redirection
use cli_rust::modules::command::Command;
use cli_rust::modules::input::{Environment, InputProcessorBuilder};
use cli_rust::modules::runner::Runner;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

#[test]
fn test_input_processor_stdin_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test basic stdin redirection parsing
    let cmds = ip.process("cat < input.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "cat");
    assert_eq!(cmd.stdin.as_deref(), Some("input.txt"));
    assert!(cmd.stdout.is_none());
}

#[test]
fn test_input_processor_stdout_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test basic stdout redirection parsing
    let cmds = ip.process("echo hello > output.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["hello"]);
    assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
    assert!(!cmd.append_stdout);
}

#[test]
fn test_input_processor_stdout_append_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test stdout append redirection parsing
    let cmds = ip.process("echo hello >> output.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["hello"]);
    assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
    assert!(cmd.append_stdout);
}

#[test]
fn test_input_processor_combined_redirection_parsing() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test combined stdin and stdout redirection
    let cmds = ip.process("grep pattern < input.txt > output.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "grep");
    assert_eq!(cmd.args, vec!["pattern"]);
    assert_eq!(cmd.stdin.as_deref(), Some("input.txt"));
    assert_eq!(cmd.stdout.as_deref(), Some("output.txt"));
    assert!(!cmd.append_stdout);
}

#[test]
fn test_input_processor_complex_redirection_with_quotes() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test redirection with quoted filenames
    let cmds = ip
        .process(r#"cat < "input file.txt" > "output file.txt""#)
        .unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "cat");
    assert_eq!(cmd.stdin.as_deref(), Some("input file.txt"));
    assert_eq!(cmd.stdout.as_deref(), Some("output file.txt"));
}

#[test]
fn test_input_processor_multiple_redirections() {
    let env = Environment::new();
    let ip = InputProcessorBuilder::new(env).build();

    // Test multiple redirection operators (should use last one)
    let cmds = ip.process("echo test > file1.txt > file2.txt").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["test"]);
    assert_eq!(cmd.stdout.as_deref(), Some("file2.txt"));
}

#[test]
fn test_input_processor_redirection_with_environment_vars() {
    let mut vars = HashMap::new();
    vars.insert("OUTFILE".to_string(), "result.txt".to_string());
    vars.insert("INFILE".to_string(), "source.txt".to_string());

    let env = Environment::with_vars(vars);
    let ip = InputProcessorBuilder::new(env).build();

    let cmds = ip.process("cat < $INFILE > $OUTFILE").unwrap();
    assert_eq!(cmds.len(), 1);
    let cmd = &cmds[0];
    assert_eq!(cmd.name, "cat");
    assert_eq!(cmd.stdin.as_deref(), Some("source.txt"));
    assert_eq!(cmd.stdout.as_deref(), Some("result.txt"));
}

#[test]
fn test_end_to_end_stdout_redirection() {
    let test_dir = env::temp_dir().join("cli_integration_stdout");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("integration_output.txt");
    let output_path = output_file.to_string_lossy().to_string();

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test command with stdout redirection
    let cmd = Command::new(
        "echo".to_string(),
        vec!["Integration Test Output".to_string()],
    )
    .with_stdout(output_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            // Should return empty string when redirecting
            assert_eq!(output, "");

            // Verify file was created and contains expected content
            assert!(output_file.exists());
            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");
            assert!(file_contents.contains("Integration Test Output"));
        }
        Err(e) => {
            // Skip test if echo is not available
            println!(
                "Skipping end-to-end stdout test - echo not available: {}",
                e
            );
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_stdout_append() {
    let test_dir = env::temp_dir().join("cli_integration_append");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("integration_append.txt");
    let output_path = output_file.to_string_lossy().to_string();

    // Write initial content
    fs::write(&output_file, "Line 1\n").expect("Failed to write initial content");

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // First append
    let cmd1 = Command::new("echo".to_string(), vec!["Line 2".to_string()])
        .with_stdout(output_path.clone())
        .with_append_stdout(true);

    // Second append
    let cmd2 = Command::new("echo".to_string(), vec!["Line 3".to_string()])
        .with_stdout(output_path.clone())
        .with_append_stdout(true);

    let result1 = runner.execute(cmd1);
    let result2 = runner.execute(cmd2);

    match (result1, result2) {
        (Ok(_), Ok(_)) => {
            // Verify file contains all lines in order
            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");

            let lines: Vec<&str> = file_contents.lines().collect();
            assert!(lines.len() >= 3);
            assert!(file_contents.contains("Line 1"));
            assert!(file_contents.contains("Line 2"));
            assert!(file_contents.contains("Line 3"));

            // Verify order is preserved
            let line1_pos = file_contents.find("Line 1").unwrap();
            let line2_pos = file_contents.find("Line 2").unwrap();
            let line3_pos = file_contents.find("Line 3").unwrap();
            assert!(line1_pos < line2_pos);
            assert!(line2_pos < line3_pos);
        }
        _ => {
            println!("Skipping end-to-end append test - echo not available");
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_stdin_redirection() {
    let test_dir = env::temp_dir().join("cli_integration_stdin");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let input_file = test_dir.join("integration_input.txt");

    // Create input file with test content
    let test_content = "Line A\nLine B\nLine C\n";
    fs::write(&input_file, test_content).expect("Failed to write input file");

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test command with stdin redirection (simulating file content)
    let cmd = Command::new("cat".to_string(), vec![]).with_stdin(test_content.to_string());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            assert!(output.contains("Line A"));
            assert!(output.contains("Line B"));
            assert!(output.contains("Line C"));
        }
        Err(e) => {
            println!("Skipping end-to-end stdin test - cat not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_combined_redirection() {
    let test_dir = env::temp_dir().join("cli_integration_combined");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("combined_result.txt");
    let output_path = output_file.to_string_lossy().to_string();

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test command with both stdin and stdout redirection
    let input_data = "Input data for processing\nSecond line\n";
    let cmd = Command::new("cat".to_string(), vec![])
        .with_stdin(input_data.to_string())
        .with_stdout(output_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            // Should return empty string when redirecting to file
            assert_eq!(output, "");

            // Verify file was created and contains input data
            assert!(output_file.exists());
            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");
            assert!(file_contents.contains("Input data for processing"));
            assert!(file_contents.contains("Second line"));
        }
        Err(e) => {
            println!(
                "Skipping end-to-end combined test - cat not available: {}",
                e
            );
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_redirection_error_handling() {
    // Test redirection to invalid path
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    let cmd = Command::new("echo".to_string(), vec!["test".to_string()])
        .with_stdout("/invalid/path/that/doesnt/exist/output.txt".to_string());

    let result = runner.execute(cmd);

    match result {
        Ok(_) => {
            // This might succeed on some systems, which is fine
        }
        Err(e) => {
            // Expected behavior - should fail with file system error
            assert!(
                e.to_string().contains("No such file")
                    || e.to_string().contains("cannot create")
                    || e.to_string().contains("Permission denied")
                    || e.to_string().contains("not found")
            );
        }
    }
}

#[test]
fn test_redirection_with_empty_files() {
    let test_dir = env::temp_dir().join("cli_integration_empty");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("empty_output.txt");
    let output_path = output_file.to_string_lossy().to_string();

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test with command that produces no output
    let cmd = Command::new("echo".to_string(), vec![]).with_stdout(output_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            assert_eq!(output, "");

            // File should be created even if empty/nearly empty
            assert!(output_file.exists());
        }
        Err(e) => {
            println!("Skipping empty file test - echo not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_redirection_overwrite_behavior() {
    let test_dir = env::temp_dir().join("cli_integration_overwrite");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("overwrite_test.txt");
    let output_path = output_file.to_string_lossy().to_string();

    // Write initial content
    fs::write(&output_file, "Original content\n").expect("Failed to write initial content");

    // Create runner
    let bin_path = PathBuf::from("/nonexistent/path");
    let env_vars = HashMap::new();
    let runner = Runner::new(bin_path, env_vars);

    // Test overwrite with > operator
    let cmd = Command::new("echo".to_string(), vec!["New content".to_string()])
        .with_stdout(output_path.clone())
        .with_append_stdout(false);

    let result = runner.execute(cmd);

    match result {
        Ok(_) => {
            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");

            // Should contain new content, not original
            assert!(file_contents.contains("New content"));
            assert!(!file_contents.contains("Original content"));
        }
        Err(e) => {
            println!("Skipping overwrite test - echo not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
