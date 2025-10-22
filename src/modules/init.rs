use std::collections::HashMap;
use std::path::PathBuf;

use crate::modules::input::{Environment, InputProcessor, InputProcessorBuilder};

#[derive(Debug, Clone)]
pub struct Init {
    /// Env variables
    pub env_vars: HashMap<String, String>,
    /// Binary path for implemented commands
    pub bin_path: PathBuf,
}

impl Init {
    pub fn new() -> Self {
        let env_vars = std::env::vars().collect();

        // Check for CLI_BIN_PATH env var
        let bin_path = if let Ok(custom_path) = std::env::var("CLI_BIN_PATH") {
            PathBuf::from(custom_path)
        } else {
            // Try default path
            let debug_path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("target")
                .join("debug");

            let release_path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("target")
                .join("release");

            // Prefer release
            if release_path.join("echo").exists() {
                release_path
            } else {
                debug_path
            }
        };

        Init { env_vars, bin_path }
    }

    /// Create a new Init with custom environment variables and binary path.
    /// Suitable for testing
    pub fn with_config(env_vars: HashMap<String, String>, bin_path: PathBuf) -> Self {
        Init { env_vars, bin_path }
    }

    /// Get an environment variable value
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }

    /// Set an environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Get all environment variables as a reference
    pub fn env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }
}

impl Default for Init {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_creation() {
        let init = Init::new();
        assert!(!init.env_vars.is_empty());
        assert!(init.bin_path.to_string_lossy().contains("target"));
    }

    #[test]
    fn test_with_config_init() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "42".to_string());
        let bin_path = PathBuf::from("/test_path/bin");

        let init = Init::with_config(env_vars, bin_path.clone());
        assert_eq!(init.get_env("TEST_VAR"), Some(&"42".to_string()));
        assert_eq!(init.bin_path, bin_path);
    }

    #[test]
    fn test_set_env() {
        let mut init = Init::new();
        init.set_env("NEW_VAR".to_string(), "new_value".to_string());
        assert_eq!(init.get_env("NEW_VAR"), Some(&"new_value".to_string()));
    }
}

pub fn build_input_processor() -> InputProcessor {
    let env = Environment::capture_current(); // реальные переменные окружения
    InputProcessorBuilder::new(env).build()
}
