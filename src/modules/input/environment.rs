use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    vars: HashMap<String, String>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
    pub fn with_vars(vars: HashMap<String, String>) -> Self {
        Self { vars }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, k: K, v: V) {
        self.vars.insert(k.into(), v.into());
    }
    pub fn remove(&mut self, k: &str) {
        self.vars.remove(k);
    }

    pub fn capture_current() -> Self {
        let vars = std::env::vars().collect::<HashMap<_, _>>();
        Self { vars }
    }
}
