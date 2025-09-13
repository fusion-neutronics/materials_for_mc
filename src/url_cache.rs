use std::path::PathBuf;
use std::fs;
use std::io::Write;
use sha2::{Sha256, Digest};

/// Get the cache directory for materials_for_mc
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

/// Generate a hash-based filename for a URL
fn url_to_filename(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    format!("{:x}.json", result)
}

/// Download a file from URL to cache directory, return the local path
pub fn download_and_cache(url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let filename = url_to_filename(url);
    let local_path = cache_dir.join(filename);
    
    // If file already exists in cache, return the path
    if local_path.exists() {
        return Ok(local_path);
    }
    
    // Download the file
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

/// Resolve a path or URL to a local file path
/// If it's a URL, download and cache it first
/// If it's a local path, return as-is
pub fn resolve_path_or_url(path_or_url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if is_url(path_or_url) {
        download_and_cache(path_or_url)
    } else {
        Ok(PathBuf::from(path_or_url))
    }
}