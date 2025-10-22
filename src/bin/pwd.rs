use std::env;
use std::io::{self, Write};
use std::process;

fn main() {
    match env::current_dir() {
        Ok(current_dir) => {
            // Convert path to string and print it
            match current_dir.to_str() {
                Some(path_str) => {
                    println!("{}", path_str);
                }
                None => {
                    eprintln!("pwd: current directory path contains invalid UTF-8");
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("pwd: error getting current directory: {}", e);
            process::exit(1);
        }
    }

    // Ensure output is flushed
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    fn get_pwd_binary_path() -> std::path::PathBuf {
        let mut path = env::current_exe().unwrap();
        // Navigate to the directory containing the built binaries
        path.pop(); // Remove the test binary name
        if path.ends_with("deps") {
            path.pop(); // Remove deps
        }
        path.push("pwd");
        path
    }

    #[test]
    fn test_pwd_basic() {
        let output = Command::new(get_pwd_binary_path())
            .output()
            .expect("Failed to execute pwd command");

        assert!(output.status.success());
        assert!(output.stderr.is_empty());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let pwd_output = stdout_str.trim();

        // Should be a valid path
        assert!(Path::new(pwd_output).is_absolute());
        assert!(!pwd_output.is_empty());
    }

    #[test]
    fn test_pwd_matches_env_current_dir() {
        let output = Command::new(get_pwd_binary_path())
            .output()
            .expect("Failed to execute pwd command");

        assert!(output.status.success());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let pwd_output = stdout_str.trim();

        // Compare with what env::current_dir() returns in the test environment
        let expected_dir = env::current_dir().expect("Failed to get current directory");
        let expected_str = expected_dir.to_str().expect("Path contains invalid UTF-8");

        assert_eq!(pwd_output, expected_str);
    }

    #[test]
    fn test_pwd_in_different_directory() {
        let test_dir = env::temp_dir().join("pwd_test_dir");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let output = Command::new(get_pwd_binary_path())
            .current_dir(&test_dir)
            .output()
            .expect("Failed to execute pwd command");

        assert!(output.status.success());
        assert!(output.stderr.is_empty());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let pwd_output = stdout_str.trim();

        // Should match the test directory we set as current_dir
        let expected_str = test_dir.to_str().expect("Path contains invalid UTF-8");
        assert_eq!(pwd_output, expected_str);

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_pwd_with_nested_directory() {
        let test_base = env::temp_dir().join("pwd_nested_test");
        let test_dir = test_base.join("level1").join("level2").join("level3");
        let _ = fs::remove_dir_all(&test_base);
        fs::create_dir_all(&test_dir).expect("Failed to create nested test directory");

        let output = Command::new(get_pwd_binary_path())
            .current_dir(&test_dir)
            .output()
            .expect("Failed to execute pwd command");

        assert!(output.status.success());
        assert!(output.stderr.is_empty());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let pwd_output = stdout_str.trim();

        // Should match the nested directory
        let expected_str = test_dir.to_str().expect("Path contains invalid UTF-8");
        assert_eq!(pwd_output, expected_str);

        // Verify it contains the nested structure
        assert!(pwd_output.contains("level1"));
        assert!(pwd_output.contains("level2"));
        assert!(pwd_output.contains("level3"));

        // Clean up
        let _ = fs::remove_dir_all(&test_base);
    }

    #[test]
    fn test_pwd_ignores_arguments() {
        // pwd should ignore any arguments passed to it
        let output = Command::new(get_pwd_binary_path())
            .arg("--help")
            .arg("-l")
            .arg("somefile")
            .output()
            .expect("Failed to execute pwd command");

        assert!(output.status.success());
        assert!(output.stderr.is_empty());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let pwd_output = stdout_str.trim();

        // Should still just print current directory, ignoring arguments
        assert!(Path::new(pwd_output).is_absolute());
        assert!(!pwd_output.is_empty());

        // Compare with baseline pwd output (no args)
        let baseline_output = Command::new(get_pwd_binary_path())
            .output()
            .expect("Failed to execute baseline pwd command");

        let baseline_str = String::from_utf8_lossy(&baseline_output.stdout);
        assert_eq!(pwd_output, baseline_str.trim());
    }

    #[test]
    fn test_pwd_output_format() {
        let output = Command::new(get_pwd_binary_path())
            .output()
            .expect("Failed to execute pwd command");

        assert!(output.status.success());

        let stdout_str = String::from_utf8_lossy(&output.stdout);

        // Should end with exactly one newline
        assert!(stdout_str.ends_with('\n'));
        let without_final_newline = stdout_str.trim_end_matches('\n');
        assert!(!without_final_newline.contains('\n')); // No other newlines

        // Should not have leading/trailing whitespace (except the final newline)
        assert!(!stdout_str.starts_with(' '));
        assert!(!stdout_str.starts_with('\t'));

        // Path should be absolute (start with /)
        assert!(without_final_newline.starts_with('/'));
    }
}
