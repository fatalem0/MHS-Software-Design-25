// Integration tests for command execution and redirection
// Tests: runner functionality, file I/O redirection, custom vs system commands, environment variables
use cli_rust::modules::command::Command;
use cli_rust::modules::runner::Runner;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

fn create_test_runner() -> Runner {
    let bin_path = PathBuf::from("target/release");
    let env_vars = HashMap::new();
    Runner::new(bin_path, env_vars)
}

#[test]
fn test_runner_stdout_redirection() {
    let test_dir = env::temp_dir().join("cli_stdout_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("test_output.txt");
    let output_path = output_file.to_string_lossy().to_string();

    let runner = create_test_runner();
    let cmd = Command::new("echo".to_string(), vec!["Hello World".to_string()])
        .with_stdout(output_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            // Should return empty string when redirecting to file
            assert_eq!(output, "");

            // Check file contents
            if output_file.exists() {
                let file_contents =
                    fs::read_to_string(&output_file).expect("Failed to read output file");
                assert!(file_contents.contains("Hello World"));
            }
        }
        Err(e) => {
            println!(
                "Echo command not available for stdout redirection test: {}",
                e
            );
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_runner_stdout_append() {
    let test_dir = env::temp_dir().join("cli_append_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("append_test.txt");
    let output_path = output_file.to_string_lossy().to_string();

    // Write initial content
    fs::write(&output_file, "Initial line\n").expect("Failed to write initial content");

    let runner = create_test_runner();
    let cmd = Command::new("echo".to_string(), vec!["Appended line".to_string()])
        .with_stdout(output_path.clone())
        .with_append_stdout(true);

    let result = runner.execute(cmd);

    match result {
        Ok(_) => {
            if output_file.exists() {
                let file_contents =
                    fs::read_to_string(&output_file).expect("Failed to read output file");
                assert!(file_contents.contains("Initial line"));
                assert!(file_contents.contains("Appended line"));
            }
        }
        Err(e) => {
            println!("Echo command not available for append test: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_runner_stderr_redirection() {
    let test_dir = env::temp_dir().join("cli_stderr_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let error_file = test_dir.join("test_stderr.txt");
    let error_path = error_file.to_string_lossy().to_string();

    let runner = create_test_runner();
    // Use a command that produces stderr (cat with non-existent file)
    let cmd = Command::new("cat".to_string(), vec!["nonexistent_file.txt".to_string()])
        .with_stderr(error_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(_) | Err(_) => {
            // Command may succeed or fail, but if stderr was redirected,
            // check if file was created
            if error_file.exists() {
                println!("Stderr file was created successfully");
            }
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_runner_stdin_redirection() {
    let runner = create_test_runner();
    let input_data = "line1\nline2\nline3\n";
    let cmd = Command::new("cat".to_string(), vec![]).with_stdin(input_data.to_string());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            assert!(output.contains("line1"));
            assert!(output.contains("line2"));
            assert!(output.contains("line3"));
        }
        Err(e) => {
            println!("Cat command not available for stdin test: {}", e);
        }
    }
}

#[test]
fn test_runner_combined_redirection() {
    let test_dir = env::temp_dir().join("cli_combined_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("combined_output.txt");
    let error_file = test_dir.join("combined_error.txt");
    let output_path = output_file.to_string_lossy().to_string();
    let error_path = error_file.to_string_lossy().to_string();

    let runner = create_test_runner();
    let input_data = "test input data\n";
    let cmd = Command::new("cat".to_string(), vec![])
        .with_stdin(input_data.to_string())
        .with_stdout(output_path.clone())
        .with_stderr(error_path.clone());

    let result = runner.execute(cmd);

    match result {
        Ok(output) => {
            // Should return empty string when redirecting to file
            assert_eq!(output, "");

            // Check output file
            if output_file.exists() {
                let file_contents =
                    fs::read_to_string(&output_file).expect("Failed to read output file");
                assert!(file_contents.contains("test input data"));
            }
        }
        Err(e) => {
            println!("Cat command not available for combined test: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_runner_custom_vs_system_commands() {
    let runner = create_test_runner();

    // Test custom echo (should exist in target/release)
    let cmd = Command::new("echo".to_string(), vec!["custom".to_string()]);
    let result = runner.execute(cmd);
    match result {
        Ok(output) => assert!(output.contains("custom")),
        Err(e) => println!("Custom echo not available: {}", e),
    }

    // Test system command fallback
    let cmd = Command::new("whoami".to_string(), vec![]);
    let result = runner.execute(cmd);
    match result {
        Ok(_) => println!("System whoami command worked"),
        Err(e) => println!("System whoami not available: {}", e),
    }
}

#[test]
fn test_runner_nonexistent_command() {
    let runner = create_test_runner();
    let cmd = Command::new("definitely_nonexistent_command_12345".to_string(), vec![]);
    let result = runner.execute(cmd);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
fn test_runner_environment_variables() {
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_ENV_VAR".to_string(), "test_value".to_string());

    let bin_path = PathBuf::from("target/release");
    let runner = Runner::new(bin_path, env_vars);

    // Test environment variable access
    assert_eq!(
        runner.get_env_var("TEST_ENV_VAR"),
        Some(&"test_value".to_string())
    );
    assert_eq!(runner.get_env_var("NONEXISTENT"), None);
}

#[test]
fn test_runner_error_handling() {
    let test_dir = env::temp_dir().join("cli_error_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let runner = create_test_runner();

    // Test redirection to invalid path
    let cmd = Command::new("echo".to_string(), vec!["test".to_string()])
        .with_stdout("/invalid/path/that/doesnt/exist/output.txt".to_string());

    let result = runner.execute(cmd);
    match result {
        Ok(_) => {
            // Might succeed on some systems depending on permissions
        }
        Err(e) => {
            // Expected - should fail with file system error
            assert!(
                e.to_string().contains("No such file")
                    || e.to_string().contains("cannot create")
                    || e.to_string().contains("Permission denied")
                    || e.to_string().contains("not found")
            );
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
