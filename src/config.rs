// Global configuration for the materials library
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global configuration for file paths and other settings
pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    Mutex::new(Config::new())
});

#[derive(Debug, Clone)]
pub struct Config {
    /// Paths to nuclide cross section JSON files (nuclide name -> file path)
    pub cross_sections: HashMap<String, String>,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Config {
            cross_sections: HashMap::new(),
        }
    }

    /// Set a cross section file path for a nuclide
    pub fn set_cross_section(&mut self, nuclide: &str, path: &str) {
        self.cross_sections.insert(nuclide.to_string(), path.to_string());
    }

    /// Get a cross section file path for a nuclide
    pub fn get_cross_section(&self, nuclide: &str) -> Option<String> {
        self.cross_sections.get(nuclide).cloned()
    }

    /// Set multiple cross section file paths at once
    pub fn set_cross_sections(&mut self, paths: HashMap<String, String>) {
        for (nuclide, path) in paths {
            self.cross_sections.insert(nuclide, path);
        }
    }

    /// Get the global configuration instance
    pub fn global() -> std::sync::MutexGuard<'static, Self> {
        CONFIG.lock().unwrap()
    }
}