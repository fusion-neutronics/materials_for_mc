// Global configuration for the materials library
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Global configuration for nuclear data file paths
pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::new()));

/// Global configuration container for the nuclear data library.
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
    /// Optional global default cross section source keyword
    pub default_cross_section: Option<String>,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Config {
            cross_sections: HashMap::new(),
            default_cross_section: None,
        }
    }

    /// Set a cross section file path for a nuclide, or set a global default if only a keyword is provided
    pub fn set_cross_section(&mut self, nuclide_or_keyword: &str, path: Option<&str>) {
        // List of acceptable keywords
        const ACCEPTABLE_KEYWORDS: &[&str] = &["tendl-21", "fendl-3.2c"];
        match path {
            Some(p) => {
                self.cross_sections.insert(nuclide_or_keyword.to_string(), p.to_string());
            }
            None => {
                // Validate keyword
                if !ACCEPTABLE_KEYWORDS.contains(&nuclide_or_keyword) {
                    panic!("Invalid cross section keyword: '{}'. Acceptable keywords are: {}", nuclide_or_keyword, ACCEPTABLE_KEYWORDS.join(", "));
                }
                self.default_cross_section = Some(nuclide_or_keyword.to_string());
            }
        }
    }

    /// Get a cross section file path for a nuclide, falling back to the global default if not set
    pub fn get_cross_section(&self, nuclide: &str) -> Option<String> {
        self.cross_sections.get(nuclide).cloned()
            .or_else(|| self.default_cross_section.clone())
    }

    /// Set multiple cross section file paths at once, or set a global keyword
    pub fn set_cross_sections<T>(&mut self, input: T)
    where
        T: IntoCrossSectionsInput,
    {
        input.apply(self);
    }

    /// Clear all cross section mappings and default
    pub fn clear(&mut self) {
        self.cross_sections.clear();
        self.default_cross_section = None;
    }
}

/// Trait to allow flexible input types for set_cross_sections
pub trait IntoCrossSectionsInput {
    fn apply(self, config: &mut Config);
}

impl IntoCrossSectionsInput for HashMap<String, String> {
    fn apply(self, config: &mut Config) {
        // List of acceptable keywords
        const ACCEPTABLE_KEYWORDS: &[&str] = &["tendl-21", "fendl-3.2c"];
        for (nuclide, path) in self {
            if ACCEPTABLE_KEYWORDS.contains(&path.as_str()) {
                // If value is a keyword, set as global default
                config.default_cross_section = Some(path.clone());
            }
            config.cross_sections.insert(nuclide, path);
        }
    }
}

impl IntoCrossSectionsInput for &str {
    fn apply(self, config: &mut Config) {
        // List of acceptable keywords
        const ACCEPTABLE_KEYWORDS: &[&str] = &["tendl-21", "fendl-3.2c"];
        if !ACCEPTABLE_KEYWORDS.contains(&self) {
            panic!("Invalid cross section keyword: '{}'. Acceptable keywords are: {}", self, ACCEPTABLE_KEYWORDS.join(", "));
        }
        config.default_cross_section = Some(self.to_string());
    }
}

impl IntoCrossSectionsInput for String {
    fn apply(self, config: &mut Config) {
        IntoCrossSectionsInput::apply(self.as_str(), config);
    }
}

impl Config {
    /// Get the global configuration instance
    pub fn global() -> std::sync::MutexGuard<'static, Self> {
        CONFIG.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}



#[cfg(test)]
mod tests {
use super::*;

#[test]
fn test_set_cross_section_global_keyword() {
    let mut config = Config::new();
    config.set_cross_section("tendl-21", None);
    assert_eq!(config.default_cross_section, Some("tendl-21".to_string()));
}

#[test]
#[should_panic(expected = "Invalid cross section keyword")]
fn test_set_cross_section_invalid_keyword() {
    let mut config = Config::new();
    config.set_cross_section("invalid-keyword", None);
}

#[test]
fn test_set_and_get_cross_section_for_nuclide() {
    let mut config = Config::new();
    config.set_cross_section("Li6", Some("/path/to/Li6.json"));
    assert_eq!(config.get_cross_section("Li6"), Some("/path/to/Li6.json".to_string()));
}

#[test]
fn test_get_cross_section_fallback_to_global() {
    let mut config = Config::new();
    config.set_cross_section("tendl-21", None);
    assert_eq!(config.get_cross_section("Fe56"), Some("tendl-21".to_string()));
}

#[test]
fn test_set_cross_section_path_to_file() {
    let mut config = Config::new();
    config.set_cross_section("Fe56", Some("../../tests/Fe56.json"));
    assert_eq!(config.get_cross_section("Fe56"), Some("../../tests/Fe56.json".to_string()));
}

#[test]
fn test_set_cross_section_single_keyword() {
    let mut config = Config::new();
    config.set_cross_section("Fe56", Some("tendl-21"));
    assert_eq!(config.get_cross_section("Fe56"), Some("tendl-21".to_string()));
}

#[test]
fn test_set_cross_sections_multiple() {
    let mut config = Config::new();
    let cross_sections = std::collections::HashMap::from([
        ("Li7".to_string(), "../../tests/Li7.json".to_string()),
        ("Li6".to_string(), "tendl-21".to_string()),
    ]);
    config.set_cross_sections(cross_sections);
    assert_eq!(config.get_cross_section("Li7"), Some("../../tests/Li7.json".to_string()));
    assert_eq!(config.get_cross_section("Li6"), Some("tendl-21".to_string()));
}

#[test]
fn test_set_cross_section_global_keyword_for_all() {
    let mut config = Config::new();
    config.set_cross_section("tendl-21", None);
    assert_eq!(config.get_cross_section("Li6"), Some("tendl-21".to_string()));
    assert_eq!(config.get_cross_section("Fe56"), Some("tendl-21".to_string()));
}

#[test]
fn test_set_cross_sections_with_string_keyword() {
    let mut config = Config::new();
    config.set_cross_sections("tendl-21");
    assert_eq!(config.default_cross_section, Some("tendl-21".to_string()));
    assert_eq!(config.get_cross_section("Li6"), Some("tendl-21".to_string()));
    assert_eq!(config.get_cross_section("Fe56"), Some("tendl-21".to_string()));
}

#[test]
fn test_set_cross_sections_with_hashmap() {
    let mut config = Config::new();
    let cross_sections = std::collections::HashMap::from([
        ("Li6".to_string(), "../../tests/Li6.json".to_string()),
        ("Fe56".to_string(), "tendl-21".to_string()),
    ]);
    config.set_cross_sections(cross_sections);
    assert_eq!(config.get_cross_section("Li6"), Some("../../tests/Li6.json".to_string()));
    assert_eq!(config.get_cross_section("Fe56"), Some("tendl-21".to_string()));
    // When a keyword is in the hashmap, it should set the global default too
    assert_eq!(config.default_cross_section, Some("tendl-21".to_string()));
}

#[test]
#[should_panic(expected = "Invalid cross section keyword")]
fn test_set_cross_sections_invalid_string_keyword() {
    let mut config = Config::new();
    config.set_cross_sections("invalid-keyword");
}
}