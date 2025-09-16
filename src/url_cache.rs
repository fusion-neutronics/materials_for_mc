use std::path::PathBuf;
use std::collections::HashMap;

#[cfg(feature = "download")]
use std::fs;
#[cfg(feature = "download")]
use std::io::Write;

/// Get the cache directory for materials_for_mc
#[cfg(feature = "download")]
pub fn get_cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?;
    
    let cache_dir = home_dir.join(".cache").join("materials_for_mc");
    
    // Create the cache directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    
    Ok(cache_dir)
}

/// Get the mapping of keywords to URL stems
fn get_keyword_url_mapping() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("tendl-21", "https://raw.githubusercontent.com/fusion-neutronics/cross_section_data_tendl_2021/refs/heads/main/tendl_2021/");
    map.insert("fendl-3.2c", "https://raw.githubusercontent.com/fusion-neutronics/cross_section_data_fendl_3.2c/refs/heads/main/fendl3.2c_data/");
    // Add more keyword mappings here as needed
    map
}

/// Check if a string is a known keyword
pub fn is_keyword(input: &str) -> bool {
    get_keyword_url_mapping().contains_key(input)
}

/// Expand a keyword to a full URL for a specific nuclide
#[cfg(feature = "download")]
pub fn expand_keyword_to_url(keyword: &str, nuclide_name: &str) -> Option<String> {
    get_keyword_url_mapping()
        .get(keyword)
        .map(|stem| format!("{}{}.json", stem, nuclide_name))
}

/// Generate a filename for cache based on keyword or URL
#[cfg(feature = "download")]
fn generate_cache_filename(source: &str, nuclide_name: &str) -> String {
    if is_keyword(source) {
        format!("{}-{}.json", source, nuclide_name)
    } else {
        // For direct URLs, still use the nuclide name
        format!("{}.json", nuclide_name)
    }
}

/// Download a file from URL to cache directory, return the local path
#[cfg(feature = "download")]
pub fn download_and_cache(url: &str, source: &str, nuclide_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let filename = generate_cache_filename(source, nuclide_name);
    let local_path = cache_dir.join(filename);
    
    // If file already exists in cache, return the path
    if local_path.exists() {
        return Ok(local_path);
    }
    
    // Download the file
    println!("Downloading URL to cache: {} -> {:?}", url, local_path);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download {}: {}", url, response.status()).into());
    }
    
    let content = response.bytes()?;
    
    // Write to cache
    let mut file = fs::File::create(&local_path)?;
    file.write_all(&content)?;
    
    Ok(local_path)
}

/// Check if a string looks like a URL (starts with http:// or https://)
pub fn is_url(path_or_url: &str) -> bool {
    path_or_url.starts_with("http://") || path_or_url.starts_with("https://")
}

/// Resolve a path, URL, or keyword to a local file path
/// If it's a keyword, expand it to URL and download
/// If it's a URL, download and cache it
/// If it's a local path, return as-is
#[cfg(feature = "download")]
pub fn resolve_path_or_url(path_url_or_keyword: &str, nuclide_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if is_keyword(path_url_or_keyword) {
        // It's a keyword, expand to URL and download
        let url = expand_keyword_to_url(path_url_or_keyword, nuclide_name)
            .ok_or_else(|| format!("Unknown keyword: {}", path_url_or_keyword))?;
        download_and_cache(&url, path_url_or_keyword, nuclide_name)
    } else if is_url(path_url_or_keyword) {
        // It's a direct URL
        download_and_cache(path_url_or_keyword, path_url_or_keyword, nuclide_name)
    } else {
        // It's a local path
        Ok(PathBuf::from(path_url_or_keyword))
    }
}

/// WASM fallback - only supports local paths, not URLs or keywords
#[cfg(not(feature = "download"))]
pub fn resolve_path_or_url(path_url_or_keyword: &str, _nuclide_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if is_keyword(path_url_or_keyword) {
        return Err("URL downloading and keywords are not supported in WASM builds. Please use local file paths.".into());
    } else if is_url(path_url_or_keyword) {
        return Err("URL downloading is not supported in WASM builds. Please use local file paths.".into());
    } else {
        // It's a local path
        Ok(PathBuf::from(path_url_or_keyword))
    }
}