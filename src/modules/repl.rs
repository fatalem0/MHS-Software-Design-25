use crate::modules::init::Init;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Repl {
    bin_path: PathBuf,
    env_vars: HashMap<String, String>,
}

impl Repl {
    pub fn new(init: &Init) -> Self {
        let bin_path = init.bin_path.clone();
        Repl { 
            bin_path,
            env_vars: init.env_vars().clone(),
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        let init = Init::new();
        Self::new(&init)
    }
}