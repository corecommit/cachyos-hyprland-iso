//! ~/.config/hypr/popups.conf: simple key=value lines for sizes and placement.
//! Edit the file and open the popup again; no rebuild needed.

use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Conf(HashMap<String, String>);

impl Conf {
    pub fn load() -> Conf {
        let mut m = HashMap::new();
        let home = env::var("HOME").unwrap_or_default();
        if let Ok(s) = fs::read_to_string(format!("{home}/.config/hypr/popups.conf")) {
            for line in s.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    m.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        Conf(m)
    }

    pub fn int(&self, key: &str, default: i32) -> i32 {
        self.0.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
}
