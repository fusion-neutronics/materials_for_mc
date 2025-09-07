// Global configuration for the materials library
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global configuration for file paths and other settings
pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    Mutex::new(Config::new())
});

/// Global configuration container for the library.
///
/// The configuration is primarily a mapping from nuclide names (e.g. "Li6")
/// to the file system path of the JSON file that stores the reaction / energy
/// data for that nuclide. Helper methods are provided to set a single path,
/// bulk insert many paths, or query the mapping.
///
/// A single global instance is exposed via the `CONFIG` static (a
/// `Lazy<Mutex<Config>>`). Most code should obtain a guard with
/// [`Config::global`] rather than accessing the mutex directly to keep usage
/// consistent and centralized.
#[derive(Debug, Clone)]
pub struct Config {
    /// Map of nuclide name -> absolute or relative path to its JSON data file.
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