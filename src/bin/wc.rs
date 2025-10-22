use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;

fn count_lines_words_bytes(content: &str) -> (usize, usize, usize) {
    let bytes = content.len();
    let lines = if content.is_empty() {
        0
    } else {
        content.lines().count()
    };
    let words = content.split_whitespace().count();

    (lines, words, bytes)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Skip program name
    let file_args: Vec<&String> = args.iter().skip(1).collect();

    if file_args.is_empty() {
        // Read from stdin if no files provided
        let mut input = String::new();
        match io::stdin().read_to_string(&mut input) {
            Ok(_) => {
                let (lines, words, bytes) = count_lines_words_bytes(&input);
                println!("{:8} {:8} {:8}", lines, words, bytes);
            }
            Err(e) => {
                eprintln!("wc: error reading from stdin: {}", e);
                process::exit(1);
            }
        }
    } else {
        let mut total_lines = 0;
        let mut total_words = 0;
        let mut total_bytes = 0;
        let mut file_count = 0;

        // Process each file
        for file_path in &file_args {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let (lines, words, bytes) = count_lines_words_bytes(&content);
                    println!("{:8} {:8} {:8} {}", lines, words, bytes, file_path);

                    total_lines += lines;
                    total_words += words;
                    total_bytes += bytes;
                    file_count += 1;
                }
                Err(e) => {
                    eprintln!("wc: {}: {}", file_path, e);
                    process::exit(1);
                }
            }
        }

        // Show totals if more than one file
        if file_count > 1 {
            println!(
                "{:8} {:8} {:8} total",
                total_lines, total_words, total_bytes
            );
        }
    }

    // Ensure output is flushed
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::process::Command;

    fn get_wc_binary_path() -> std::path::PathBuf {
        let mut path = env::current_exe().unwrap();
        // Navigate to the directory containing the built binaries
        path.pop(); // Remove the test binary name
        if path.ends_with("deps") {
            path.pop(); // Remove deps
        }
        path.push("wc");
        path
    }

    #[test]
    fn test_count_lines_words_bytes() {
        let content = "Hello world\nThis is a test\n";
        let (lines, words, bytes) = count_lines_words_bytes(content);
        assert_eq!(lines, 2);
        assert_eq!(words, 6);
        assert_eq!(bytes, content.len());
    }

    #[test]
    fn test_count_empty_content() {
        let content = "";
        let (lines, words, bytes) = count_lines_words_bytes(content);
        assert_eq!(lines, 0);
        assert_eq!(words, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn test_count_single_line_no_newline() {
        let content = "Hello world";
        let (lines, words, bytes) = count_lines_words_bytes(content);
        assert_eq!(lines, 1);
        assert_eq!(words, 2);
        assert_eq!(bytes, 11);
    }

    #[test]
    fn test_wc_single_file() {
        let test_dir = env::temp_dir().join("wc_test_single");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let test_file = test_dir.join("test.txt");
        let test_content = "Hello world\nThis is a test file\nWith multiple lines\n";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        let output = Command::new(get_wc_binary_path())
            .arg(&test_file)
            .output()
            .expect("Failed to execute wc command");

        assert!(output.status.success());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout_str.split_whitespace().collect();

        // Should have format: lines words bytes filename
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "3"); // 3 lines
        assert_eq!(parts[1], "10"); // 10 words
        assert_eq!(parts[2], &test_content.len().to_string()); // bytes
        assert!(parts[3].contains("test.txt"));

        assert!(output.stderr.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_wc_multiple_files() {
        let test_dir = env::temp_dir().join("wc_test_multiple");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.txt");
        let content1 = "Line 1\nLine 2\n";
        let content2 = "Single line";

        fs::write(&file1, content1).expect("Failed to write file1");
        fs::write(&file2, content2).expect("Failed to write file2");

        let output = Command::new(get_wc_binary_path())
            .arg(&file1)
            .arg(&file2)
            .output()
            .expect("Failed to execute wc command");

        assert!(output.status.success());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout_str.trim().split('\n').collect();

        // Should have 3 lines: file1, file2, total
        assert_eq!(lines.len(), 3);

        // Check that total line exists
        assert!(lines[2].contains("total"));

        assert!(output.stderr.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_wc_nonexistent_file() {
        let output = Command::new(get_wc_binary_path())
            .arg("nonexistent_file_12345.txt")
            .output()
            .expect("Failed to execute wc command");

        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        assert!(stderr_str.contains("wc:"));
        assert!(stderr_str.contains("nonexistent_file_12345.txt"));
    }

    #[test]
    fn test_wc_empty_file() {
        let test_dir = env::temp_dir().join("wc_test_empty");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let empty_file = test_dir.join("empty.txt");
        fs::write(&empty_file, "").expect("Failed to create empty file");

        let output = Command::new(get_wc_binary_path())
            .arg(&empty_file)
            .output()
            .expect("Failed to execute wc command");

        assert!(output.status.success());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout_str.split_whitespace().collect();

        assert_eq!(parts[0], "0"); // 0 lines
        assert_eq!(parts[1], "0"); // 0 words
        assert_eq!(parts[2], "0"); // 0 bytes

        assert!(output.stderr.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_wc_from_stdin() {
        let input = "Line 1\nLine 2\nLine 3\n";

        let output = Command::new(get_wc_binary_path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn wc command");

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

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout_str.split_whitespace().collect();

        assert_eq!(parts.len(), 3); // lines words bytes (no filename for stdin)
        assert_eq!(parts[0], "3"); // 3 lines
        assert_eq!(parts[1], "6"); // 6 words
        assert_eq!(parts[2], &input.len().to_string()); // bytes

        assert!(output.stderr.is_empty());
    }
}
