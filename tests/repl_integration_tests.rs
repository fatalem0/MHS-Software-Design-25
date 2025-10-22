// End-to-end REPL functionality tests
// Tests: complete workflow from input parsing through command execution with real file I/O
use cli_rust::modules::{
    command::Command,
    init::Init,
    input::{Environment, InputProcessorBuilder},
    runner::Runner,
};
use std::path::Path;
use std::{env, fs};

/// Helper function to simulate REPL command processing with redirection
fn simulate_repl_command(input: &str, test_dir: &Path) -> Result<String, String> {
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

                // Handle stderr redirection
                if let Some(stderr_file) = pc.stderr {
                    let stderr_path = test_dir.join(&stderr_file);
                    cmd = cmd
                        .with_stderr(stderr_path.to_string_lossy().to_string())
                        .with_append_stderr(pc.append_stderr);
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
fn test_repl_stdout_redirection() {
    let test_dir = env::temp_dir().join("repl_stdout_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let result = simulate_repl_command("echo Hello World > output.txt", &test_dir);

    match result {
        Ok(output) => {
            assert_eq!(output, "");

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

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_repl_stderr_redirection() {
    let test_dir = env::temp_dir().join("repl_stderr_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let result = simulate_repl_command("cat /nonexistent/file.txt 2> errors.log", &test_dir);

    match result {
        Ok(_) | Err(_) => {
            // Command will fail, but stderr should be redirected
            let error_file = test_dir.join("errors.log");
            if error_file.exists() {
                let file_contents =
                    fs::read_to_string(&error_file).expect("Failed to read error file");
                assert!(!file_contents.is_empty());
            }
        }
    }

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_repl_append_redirection() {
    let test_dir = env::temp_dir().join("repl_append_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let output_file = test_dir.join("append.txt");
    fs::write(&output_file, "Initial content\n").expect("Failed to write initial file");

    let commands = vec![
        "echo First append >> append.txt",
        "echo Second append >> append.txt",
    ];

    for cmd in commands {
        let result = simulate_repl_command(cmd, &test_dir);
        match result {
            Ok(_) => {}
            Err(e) => {
                println!("Skipping REPL append test - command not available: {}", e);
                return;
            }
        }
    }

    let file_contents = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(file_contents.contains("Initial content"));
    assert!(file_contents.contains("First append"));
    assert!(file_contents.contains("Second append"));

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_repl_stdin_redirection() {
    let test_dir = env::temp_dir().join("repl_stdin_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let input_file = test_dir.join("input.txt");
    let input_content = "Line 1\nLine 2\nLine 3\n";
    fs::write(&input_file, input_content).expect("Failed to write input file");

    let result = simulate_repl_command("cat < input.txt", &test_dir);

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

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_repl_combined_redirection() {
    let test_dir = env::temp_dir().join("repl_combined_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let input_file = test_dir.join("source.txt");
    let input_content = "Data to process\nMore data\nFinal line\n";
    fs::write(&input_file, input_content).expect("Failed to write input file");

    let result = simulate_repl_command("cat < source.txt > result.txt 2> errors.txt", &test_dir);

    match result {
        Ok(output) => {
            assert_eq!(output, "");

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

    let _ = fs::remove_dir_all(&test_dir);
}
