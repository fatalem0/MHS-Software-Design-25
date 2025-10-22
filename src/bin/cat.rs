use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Skip program name
    let file_args: Vec<&String> = args.iter().skip(1).collect();

    if file_args.is_empty() {
        // Read from stdin if no files provided
        let mut input = String::new();
        match io::stdin().read_to_string(&mut input) {
            Ok(_) => print!("{}", input),
            Err(e) => {
                eprintln!("cat: error reading from stdin: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Read and print each file
        for file_path in file_args {
            match fs::read_to_string(file_path) {
                Ok(content) => print!("{}", content),
                Err(e) => {
                    eprintln!("cat: {}: {}", file_path, e);
                    process::exit(1);
                }
            }
        }
    }

    // Ensure output is flushed
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process::Command;

    fn get_cat_binary_path() -> std::path::PathBuf {
        let mut path = env::current_exe().unwrap();
        // Navigate to the directory containing the built binaries
        path.pop(); // Remove the test binary name
        if path.ends_with("deps") {
            path.pop(); // Remove deps
        }
        path.push("cat");
        path
    }

    #[test]
    fn test_cat_single_file() {
        let test_dir = env::temp_dir().join("cat_test_single");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let test_file = test_dir.join("test.txt");
        let test_content = "Hello, World!\nThis is a test file.\n";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        let output = Command::new(get_cat_binary_path())
            .arg(&test_file)
            .output()
            .expect("Failed to execute cat command");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), test_content);
        assert!(output.stderr.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cat_multiple_files() {
        let test_dir = env::temp_dir().join("cat_test_multiple");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.txt");
        let content1 = "First file content\n";
        let content2 = "Second file content\n";

        fs::write(&file1, content1).expect("Failed to write file1");
        fs::write(&file2, content2).expect("Failed to write file2");

        let output = Command::new(get_cat_binary_path())
            .arg(&file1)
            .arg(&file2)
            .output()
            .expect("Failed to execute cat command");

        assert!(output.status.success());
        let expected_output = format!("{}{}", content1, content2);
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected_output);
        assert!(output.stderr.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cat_nonexistent_file() {
        let output = Command::new(get_cat_binary_path())
            .arg("nonexistent_file_12345.txt")
            .output()
            .expect("Failed to execute cat command");

        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        assert!(stderr_str.contains("cat:"));
        assert!(stderr_str.contains("nonexistent_file_12345.txt"));
    }

    #[test]
    fn test_cat_empty_file() {
        let test_dir = env::temp_dir().join("cat_test_empty");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let empty_file = test_dir.join("empty.txt");
        fs::write(&empty_file, "").expect("Failed to create empty file");

        let output = Command::new(get_cat_binary_path())
            .arg(&empty_file)
            .output()
            .expect("Failed to execute cat command");

        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cat_from_stdin() {
        let input = "Input from stdin\nMultiple lines\n";

        let output = Command::new(get_cat_binary_path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn cat command");

        let mut child = output;

        // Write to stdin
        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), input);
        assert!(output.stderr.is_empty());
    }
}
