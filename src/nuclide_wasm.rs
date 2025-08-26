use wasm_bindgen::prelude::*;
use js_sys::Array;
use std::sync::Arc;
use crate::nuclide::{Nuclide, read_nuclide_from_json_str};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Store the string content of nuclide data for WASM environment
static NUCLIDE_JSON_CONTENT: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Global cache for WASM nuclides to avoid reloading
static WASM_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Arc<Nuclide>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Set the JSON content for a nuclide - this is used in WASM environment
pub fn set_nuclide_json_content(name: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut json_content = NUCLIDE_JSON_CONTENT.lock().unwrap();
    json_content.insert(name.to_string(), content.to_string());
    Ok(())
}

// Get a nuclide from the WASM cache or load it from the JSON content
pub fn get_or_load_nuclide_wasm(
    nuclide_name: &str, 
    _json_path_map: &HashMap<String, String>
) -> Result<Arc<Nuclide>, Box<dyn std::error::Error>> {
    // Try to get from cache first
    {
        let cache = WASM_NUCLIDE_CACHE.lock().unwrap();
        if let Some(nuclide) = cache.get(nuclide_name) {
            return Ok(Arc::clone(nuclide));
        }
    }
    
    // In WASM, try to get the JSON content from memory
    let content = {
        let json_content = NUCLIDE_JSON_CONTENT.lock().unwrap();
        json_content.get(nuclide_name).cloned()
    };
    
    if let Some(content) = content {
        // If we have content, parse it
        let nuclide = read_nuclide_from_json_str(&content)?;
        let arc_nuclide = Arc::new(nuclide);
        
        // Store in cache
        {
            let mut cache = WASM_NUCLIDE_CACHE.lock().unwrap();
            cache.insert(nuclide_name.to_string(), Arc::clone(&arc_nuclide));
        }
        
        return Ok(arc_nuclide);
    } else {
        return Err(format!("No JSON content provided for nuclide '{}' in WASM environment", nuclide_name).into());
    }
}

#[wasm_bindgen]
pub struct WasmNuclide {
    inner: Arc<Nuclide>,
}

#[wasm_bindgen]
impl WasmNuclide {
    #[wasm_bindgen]
    pub fn load_from_json(name: &str, json_path: &str) -> Result<WasmNuclide, JsValue> {
        let mut map = std::collections::HashMap::new();
        map.insert(name.to_string(), json_path.to_string());
        
        match get_or_load_nuclide_wasm(name, &map) {
            Ok(nuclide) => Ok(WasmNuclide { inner: nuclide }),
            Err(e) => Err(JsValue::from_str(&format!("Failed to load nuclide: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn load_from_json_str(name: &str, json_content: &str) -> Result<WasmNuclide, JsValue> {
        // First set the JSON content in the global storage
        match set_nuclide_json_content(name, json_content) {
            Ok(_) => {},
            Err(e) => return Err(JsValue::from_str(&format!("Failed to store nuclide JSON: {:?}", e))),
        }
        
        // Then load it using the regular WASM loading method
        let mut map = std::collections::HashMap::new();
        map.insert(name.to_string(), "in-memory".to_string());
        
        match get_or_load_nuclide_wasm(name, &map) {
            Ok(nuclide) => Ok(WasmNuclide { inner: nuclide }),
            Err(e) => Err(JsValue::from_str(&format!("Failed to parse nuclide JSON: {:?}", e))),
        }
    }
    
    #[wasm_bindgen]
    pub fn get_name(&self) -> String {
        self.inner.name.clone().unwrap_or_else(|| "Unknown".to_string())
    }
    
    #[wasm_bindgen]
    pub fn get_available_temperatures(&self) -> Array {
        let temps = if !self.inner.available_temperatures.is_empty() {
            self.inner.available_temperatures.clone()
        } else {
            self.inner.reactions.keys().cloned().collect::<Vec<String>>()
        };
        temps.into_iter().map(|t| JsValue::from_str(&t)).collect::<Array>()
    }
    
    #[wasm_bindgen]
    pub fn get_available_reactions(&self, temperature: &str) -> Result<Array, JsValue> {
        match self.inner.reactions.get(temperature) {
            Some(reactions) => {
                let mt_numbers = reactions.keys().cloned().collect::<Vec<i32>>();
                Ok(mt_numbers.into_iter()
                    .map(|mt| JsValue::from(mt))
                    .collect::<Array>())
            },
            None => Err(JsValue::from_str(&format!("Temperature {} not found", temperature))),
        }
    }
}

#[wasm_bindgen]
pub fn wasm_read_nuclide_from_json(name: &str, json_path: &str) -> Result<WasmNuclide, JsValue> {
    WasmNuclide::load_from_json(name, json_path)
}

#[wasm_bindgen]
pub fn wasm_read_nuclide_from_json_str(name: &str, json_content: &str) -> Result<WasmNuclide, JsValue> {
    WasmNuclide::load_from_json_str(name, json_content)
}

#[wasm_bindgen]
pub fn wasm_set_nuclide_data(name: &str, json_content: &str) -> Result<(), JsValue> {
    match set_nuclide_json_content(name, json_content) {
        Ok(_) => Ok(()),
        Err(e) => Err(JsValue::from_str(&format!("Failed to set nuclide data: {:?}", e))),
    }
}
