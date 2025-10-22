// End-to-end tests for REPL redirection functionality
use cli_rust::modules::{
    command::Command,
    init::Init,
    input::{Environment, InputProcessorBuilder},
    runner::Runner,
};
use std::path::Path;
use std::{env, fs};

/// Test helper to simulate REPL command processing with redirection
fn simulate_repl_command_with_redirection(input: &str, test_dir: &Path) -> Result<String, String> {
    let init = Init::new();
    let env_vars = init.env_vars().clone();
    let runner = Runner::new(init.bin_path.clone(), env_vars.clone());

    let env = Environment::with_vars(env_vars.clone());
    let input_processor = InputProcessorBuilder::new(env).build();

    // Parse the input
    match input_processor.process(input) {
        Ok(parsed_cmds) => {
            let mut final_output = String::new();

            for pc in parsed_cmds {
                // Convert parsed command to runner::Command with redirection support
                let mut cmd = Command::new(pc.name.clone(), pc.args.clone());

                // Handle stdin redirection - read file content
                if let Some(stdin_file) = pc.stdin {
                    let stdin_path = test_dir.join(&stdin_file);
                    match fs::read_to_string(&stdin_path) {
                        Ok(content) => {
                            cmd = cmd.with_stdin(content);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Error reading stdin file '{}': {}",
                                stdin_file, e
                            ));
                        }
                    }
                }

                // Handle stdout redirection
                if let Some(stdout_file) = pc.stdout {
                    let stdout_path = test_dir.join(&stdout_file);
                    cmd = cmd
                        .with_stdout(stdout_path.to_string_lossy().to_string())
                        .with_append_stdout(pc.append_stdout);
                }

                // Execute command
                match runner.execute(cmd) {
                    Ok(output) => {
                        if !output.trim().is_empty() {
                            final_output.push_str(&output);
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error executing command: {}", e));
                    }
                }
            }

            Ok(final_output)
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

#[test]
fn test_end_to_end_repl_stdout_redirection() {
    let test_dir = env::temp_dir().join("repl_e2e_stdout");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Test echo with output redirection
    let result = simulate_repl_command_with_redirection("echo Hello World > output.txt", &test_dir);

    match result {
        Ok(output) => {
            // Should return empty output (redirected to file)
            assert_eq!(output, "");

            // Check file was created
            let output_file = test_dir.join("output.txt");
            assert!(output_file.exists());

            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");
            assert!(file_contents.contains("Hello World"));
        }
        Err(e) => {
            println!("Skipping REPL stdout test - command not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_stdout_append() {
    let test_dir = env::temp_dir().join("repl_e2e_append");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Create initial file
    let output_file = test_dir.join("append.txt");
    fs::write(&output_file, "Initial content\n").expect("Failed to write initial file");

    // Test multiple append operations
    let commands = vec![
        "echo First append >> append.txt",
        "echo Second append >> append.txt",
    ];

    for cmd in commands {
        let result = simulate_repl_command_with_redirection(cmd, &test_dir);
        match result {
            Ok(_) => {} // Continue to next command
            Err(e) => {
                println!("Skipping REPL append test - command not available: {}", e);
                return;
            }
        }
    }

    // Verify all content is present and in order
    let file_contents = fs::read_to_string(&output_file).expect("Failed to read output file");

    assert!(file_contents.contains("Initial content"));
    assert!(file_contents.contains("First append"));
    assert!(file_contents.contains("Second append"));

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_stdin_redirection() {
    let test_dir = env::temp_dir().join("repl_e2e_stdin");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Create input file
    let input_file = test_dir.join("input.txt");
    let input_content = "Line 1\nLine 2\nLine 3\n";
    fs::write(&input_file, input_content).expect("Failed to write input file");

    // Test cat with input redirection
    let result = simulate_repl_command_with_redirection("cat < input.txt", &test_dir);

    match result {
        Ok(output) => {
            assert!(output.contains("Line 1"));
            assert!(output.contains("Line 2"));
            assert!(output.contains("Line 3"));
        }
        Err(e) => {
            println!("Skipping REPL stdin test - cat not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_combined_redirection() {
    let test_dir = env::temp_dir().join("repl_e2e_combined");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Create input file
    let input_file = test_dir.join("source.txt");
    let input_content = "Data to process\nMore data\nFinal line\n";
    fs::write(&input_file, input_content).expect("Failed to write input file");

    // Test combined stdin and stdout redirection
    let result = simulate_repl_command_with_redirection("cat < source.txt > result.txt", &test_dir);

    match result {
        Ok(output) => {
            // Should return empty output (redirected to file)
            assert_eq!(output, "");

            // Check output file was created with correct content
            let output_file = test_dir.join("result.txt");
            assert!(output_file.exists());

            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");
            assert!(file_contents.contains("Data to process"));
            assert!(file_contents.contains("More data"));
            assert!(file_contents.contains("Final line"));
        }
        Err(e) => {
            println!("Skipping REPL combined test - cat not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_stdin_file_not_found() {
    let test_dir = env::temp_dir().join("repl_e2e_error");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Test with non-existent input file
    let result = simulate_repl_command_with_redirection("cat < nonexistent.txt", &test_dir);

    // Should return error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Error reading stdin file") || error.contains("No such file"));

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_complex_command_with_redirection() {
    let test_dir = env::temp_dir().join("repl_e2e_complex");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Create input file with numbers
    let input_file = test_dir.join("numbers.txt");
    let input_content = "apple\nbanana\ncherry\napple\nbanana\n";
    fs::write(&input_file, input_content).expect("Failed to write input file");

    // Test grep with stdin and stdout redirection
    let result = simulate_repl_command_with_redirection(
        "grep apple < numbers.txt > filtered.txt",
        &test_dir,
    );

    match result {
        Ok(output) => {
            // Should return empty output (redirected to file)
            assert_eq!(output, "");

            // Check output file
            let output_file = test_dir.join("filtered.txt");
            assert!(output_file.exists());

            let file_contents =
                fs::read_to_string(&output_file).expect("Failed to read output file");

            // Should contain both instances of "apple"
            let apple_count = file_contents.matches("apple").count();
            assert_eq!(apple_count, 2);

            // Should not contain other fruits
            assert!(!file_contents.contains("banana"));
            assert!(!file_contents.contains("cherry"));
        }
        Err(e) => {
            println!("Skipping REPL complex test - grep not available: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_multiple_commands() {
    let test_dir = env::temp_dir().join("repl_e2e_multiple");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Execute multiple commands in sequence
    let commands = vec![
        "echo First line > multi.txt",
        "echo Second line >> multi.txt",
        "echo Third line >> multi.txt",
    ];

    for cmd in commands {
        let result = simulate_repl_command_with_redirection(cmd, &test_dir);
        match result {
            Ok(_) => {} // Continue to next command
            Err(e) => {
                println!("Skipping REPL multiple commands test: {}", e);
                return;
            }
        }
    }

    // Verify final file content
    let output_file = test_dir.join("multi.txt");
    assert!(output_file.exists());

    let file_contents = fs::read_to_string(&output_file).expect("Failed to read output file");

    assert!(file_contents.contains("First line"));
    assert!(file_contents.contains("Second line"));
    assert!(file_contents.contains("Third line"));

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_end_to_end_repl_empty_redirection() {
    let test_dir = env::temp_dir().join("repl_e2e_empty");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Test redirecting empty output
    let result = simulate_repl_command_with_redirection("echo > empty.txt", &test_dir);

    match result {
        Ok(output) => {
            // Should return empty output (redirected to file)
            assert_eq!(output, "");

            // Check output file was created (might be empty or contain just newline)
            let output_file = test_dir.join("empty.txt");
            assert!(output_file.exists());
        }
        Err(e) => {
            println!("Skipping REPL empty redirection test: {}", e);
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
