use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub seen_urls: HashSet<String>,
}

impl State {
    pub fn load(path: &Path) -> Self {
        let Ok(data) = std::fs::read_to_string(path) else {
            return State::default();
        };
        serde_json::from_str(&data).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, data)
    }

    pub fn mark_seen(&mut self, url: &str) {
        self.seen_urls.insert(url.to_string());
    }

    pub fn is_new(&self, url: &str) -> bool {
        !self.seen_urls.contains(url)
    }
}
