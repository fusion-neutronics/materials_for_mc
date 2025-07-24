use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use crate::data::{NATURAL_ABUNDANCE, ELEMENT_NUCLIDES};

#[wasm_bindgen]
pub fn natural_abundance() -> JsValue {
    let map: std::collections::HashMap<String, f64> = NATURAL_ABUNDANCE.iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
    to_value(&map).unwrap()
}

#[wasm_bindgen]
pub fn element_nuclides() -> JsValue {
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (element, nuclides) in ELEMENT_NUCLIDES.iter() {
        let mut sorted_nuclides: Vec<String> = nuclides.iter().map(|n| n.to_string()).collect();
        sorted_nuclides.sort();
        map.insert((*element).to_string(), sorted_nuclides);
    }
    to_value(&map).unwrap()
}
