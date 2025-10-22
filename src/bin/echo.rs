use std::env;

/// Echo command implementation as a standalone executable
/// Prints all arguments separated by spaces to stdout
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!();
    } else {
        println!("{}", args.join(" "));
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    #[derive(Debug, PartialEq)]
    struct CommandResult {
        stdout: String,
        stderr: String,
        exit_code: i32,
    }

    fn get_echo_binary_path() -> PathBuf {
        let debug_path = PathBuf::from("target/debug/echo");
        let release_path = PathBuf::from("target/release/echo");

        if release_path.exists() {
            release_path
        } else {
            debug_path
        }
    }

    fn run_echo_binary(args: Vec<&str>) -> CommandResult {
        let echo_path = get_echo_binary_path();
        let output = Command::new(echo_path)
            .args(args)
            .output()
            .expect("Failed to execute echo command");

        CommandResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }

    #[test]
    fn test_echo_no_arguments() {
        let result = run_echo_binary(vec![]);
        assert_eq!(result.stdout, "\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_single_word() {
        let result = run_echo_binary(vec!["hello"]);
        assert_eq!(result.stdout, "hello\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_multiple_words() {
        let result = run_echo_binary(vec!["hello", "world", "rust"]);
        assert_eq!(result.stdout, "hello world rust\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_single_character() {
        let result = run_echo_binary(vec!["a"]);
        assert_eq!(result.stdout, "a\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_empty_string() {
        let result = run_echo_binary(vec![""]);
        assert_eq!(result.stdout, "\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_whitespace_string() {
        let result = run_echo_binary(vec!["   "]);
        assert_eq!(result.stdout, "   \n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_unicode_characters() {
        let result = run_echo_binary(vec!["🦀"]);
        assert_eq!(result.stdout, "🦀\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_very_long_string() {
        let long_string = "a".repeat(1000);
        let result = run_echo_binary(vec![&long_string]);
        assert_eq!(result.stdout, format!("{}\n", long_string));
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_newline_characters() {
        let result = run_echo_binary(vec!["line1\\nline2"]);
        assert_eq!(result.stdout, "line1\\nline2\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_quotes_as_arguments() {
        let result = run_echo_binary(vec!["'quoted'", "\"double\""]);
        assert_eq!(result.stdout, "'quoted' \"double\"\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_with_dashes() {
        let result = run_echo_binary(vec!["-n", "--help", "-"]);
        assert_eq!(result.stdout, "-n --help -\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_special_characters() {
        let result = run_echo_binary(vec!["@#$%^&*()"]);
        assert_eq!(result.stdout, "@#$%^&*()\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_echo_mixed_arguments() {
        let result = run_echo_binary(vec!["hello", "123", "🦀", "world"]);
        assert_eq!(result.stdout, "hello 123 🦀 world\n");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }
}
