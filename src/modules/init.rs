use std::collections::HashMap;
use std::path::PathBuf;

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
