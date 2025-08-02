use wasm_bindgen::prelude::*;
use js_sys::{Array, JSON};
use serde::{Serialize, Deserialize};
use crate::nuclide::Reaction;

#[wasm_bindgen]
pub struct WasmReaction {
    inner: Reaction,
}

#[derive(Serialize, Deserialize)]
struct ReactionData {
    threshold_idx: usize,
    cross_section: Vec<f64>,
    interpolation: Vec<i32>,
    energy: Vec<f64>,
}

#[wasm_bindgen]
impl WasmReaction {
    #[wasm_bindgen(constructor)]
    pub fn new(threshold_idx: usize) -> Self {
        WasmReaction {
            inner: Reaction {
                threshold_idx,
                cross_section: Vec::new(),
                interpolation: Vec::new(),
                energy: Vec::new(),
                mt_number: 0, // Default MT number
            },
        }
    }
    
    #[wasm_bindgen]
    pub fn set_cross_section(&mut self, cross_section: Vec<f64>) {
        self.inner.cross_section = cross_section;
    }
    
    #[wasm_bindgen]
    pub fn set_interpolation(&mut self, interpolation: Vec<i32>) {
        self.inner.interpolation = interpolation;
    }
    
    #[wasm_bindgen]
    pub fn set_energy(&mut self, energy: Vec<f64>) {
        self.inner.energy = energy;
    }
    
    #[wasm_bindgen]
    pub fn get_threshold_idx(&self) -> usize {
        self.inner.threshold_idx
    }
    
    #[wasm_bindgen]
    pub fn get_cross_section(&self) -> Array {
        self.inner.cross_section.iter()
            .map(|&x| JsValue::from_f64(x))
            .collect::<Array>()
    }
    
    #[wasm_bindgen]
    pub fn get_interpolation(&self) -> Array {
        self.inner.interpolation.iter()
            .map(|&x| JsValue::from_f64(x as f64))
            .collect::<Array>()
    }
    
    #[wasm_bindgen]
    pub fn get_energy(&self) -> Array {
        self.inner.energy.iter()
            .map(|&x| JsValue::from_f64(x))
            .collect::<Array>()
    }
    
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let data = ReactionData {
            threshold_idx: self.inner.threshold_idx,
            cross_section: self.inner.cross_section.clone(),
            interpolation: self.inner.interpolation.clone(),
            energy: self.inner.energy.clone(),
        };
        
        let serialized = serde_json::to_string(&data)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        
        Ok(JSON::parse(&serialized)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?)
    }
}
