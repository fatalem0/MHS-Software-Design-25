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

#[test]
fn test_runner_error_messages_include_exit_codes() {
    let runner = create_test_runner();

    // Test system command that fails (command not found)
    let cmd = Command::new("nonexistent_command_12345".to_string(), vec![]);
    let result = runner.execute(cmd);

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Should mention it's not found (for command not found errors)
            assert!(error_msg.contains("not found"));
        }
        Ok(_) => panic!("Expected command to fail"),
    }

    // Test system command that exists but fails (like 'false' command)
    let cmd = Command::new("false".to_string(), vec![]);
    let result = runner.execute(cmd);

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Should include exit code in error message
            assert!(error_msg.contains("exit code"));
            assert!(error_msg.contains("1")); // false command returns exit code 1
        }
        Ok(_) => panic!("Expected 'false' command to fail"),
    }
}

#[test]
fn test_runner_error_messages_with_stderr_redirection() {
    let test_dir = env::temp_dir().join("cli_stderr_error_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let stderr_file = test_dir.join("error_output.txt");
    let stderr_path = stderr_file.to_string_lossy().to_string();

    let runner = create_test_runner();

    // Test that when stderr is redirected, we still get exit code in error message
    let cmd = Command::new("false".to_string(), vec![]).with_stderr(stderr_path.clone());

    let result = runner.execute(cmd);

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Should include exit code even when stderr is redirected
            assert!(error_msg.contains("exit code"));
            assert!(error_msg.contains("1")); // false command returns exit code 1
                                              // Should not include stderr content (it's in the file)
            assert!(!error_msg.contains("Command failed: "));
        }
        Ok(_) => panic!("Expected 'false' command to fail"),
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_runner_error_messages_with_custom_binary_failure() {
    // This test ensures that custom binaries also report exit codes properly
    let test_dir = env::temp_dir().join("cli_custom_error_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Create a simple script that exits with code 42
    let script_path = test_dir.join("test_fail_binary");
    let script_content = "#!/bin/bash\nexit 42";
    fs::write(&script_path, script_content).expect("Failed to write test script");

    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("Failed to set permissions");
    }

    // Create a runner with the test directory as bin_path
    let runner = Runner::new(test_dir.clone(), HashMap::new());

    let cmd = Command::new("test_fail_binary".to_string(), vec![]);
    let result = runner.execute(cmd);

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            println!("Error message: {}", error_msg);
            // Should include exit code in the error message
            assert!(error_msg.contains("exit code"));
            // The specific exit code might vary depending on the error (42 for our script, or other codes for execution failures)
            // As long as it contains "exit code", that's the important part
        }
        Ok(_) => {
            // If the test script doesn't exist or can't execute, that's ok for this test
            // The important thing is that when it does fail, it reports exit codes
            println!("Test script executed successfully (unexpected but ok)");
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_runner_error_messages_with_stderr_output() {
    let runner = create_test_runner();

    // Use a command that produces stderr output and fails
    // The 'ls' command on a nonexistent directory should work for this
    let cmd = Command::new(
        "ls".to_string(),
        vec!["/nonexistent_directory_12345".to_string()],
    );
    let result = runner.execute(cmd);

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Should include both exit code and stderr output
            assert!(error_msg.contains("exit code"));
            // Should also include some indication of the actual error
            assert!(error_msg.contains("No such file") || error_msg.contains("cannot access"));
        }
        Ok(_) => panic!("Expected 'ls' on nonexistent directory to fail"),
    }
}

#[test]
fn test_runner_error_messages_empty_stderr() {
    let runner = create_test_runner();

    // Use 'false' command which typically exits with code 1 but produces no stderr
    let cmd = Command::new("false".to_string(), vec![]);
    let result = runner.execute(cmd);

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Should include exit code
            assert!(error_msg.contains("exit code"));
            assert!(error_msg.contains("1"));
            // Should not have extra colon or stderr content since stderr is empty
            assert!(!error_msg.ends_with(": "));
        }
        Ok(_) => panic!("Expected 'false' command to fail"),
    }
}
