use wasm_bindgen::prelude::*;
use js_sys::Array;
use std::sync::Arc;
use crate::nuclide::{Nuclide, get_or_load_nuclide, read_nuclide_from_json_str};
use std::collections::HashMap;

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
        
        match get_or_load_nuclide(name, &map) {
            Ok(nuclide) => Ok(WasmNuclide { inner: nuclide }),
            Err(e) => Err(JsValue::from_str(&format!("Failed to load nuclide: {:?}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn load_from_json_str(name: &str, json_content: &str) -> Result<WasmNuclide, JsValue> {
        match read_nuclide_from_json_str(json_content) {
            Ok(nuclide) => {
                // Create an Arc<Nuclide> and return a WasmNuclide
                Ok(WasmNuclide { inner: Arc::new(nuclide) })
            },
            Err(e) => Err(JsValue::from_str(&format!("Failed to parse nuclide JSON: {:?}", e))),
        }
    }
    
    #[wasm_bindgen]
    pub fn get_name(&self) -> String {
        self.inner.name.clone().unwrap_or_else(|| "Unknown".to_string())
    }
    
    #[wasm_bindgen]
    pub fn get_available_temperatures(&self) -> Array {
        let temps = self.inner.reactions.keys().cloned().collect::<Vec<String>>();
        temps.into_iter()
            .map(|t| JsValue::from_str(&t))
            .collect::<Array>()
    }
    
    #[wasm_bindgen]
    pub fn get_available_reactions(&self, temperature: &str) -> Result<Array, JsValue> {
        match self.inner.reactions.get(temperature) {
            Some(reactions) => {
                let mt_numbers = reactions.keys().cloned().collect::<Vec<String>>();
                Ok(mt_numbers.into_iter()
                    .map(|mt| JsValue::from_str(&mt))
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
