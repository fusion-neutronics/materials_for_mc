use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use crate::data::NATURAL_ABUNDANCE;

#[wasm_bindgen]
pub fn natural_abundance() -> JsValue {
    let map: std::collections::HashMap<String, f64> = NATURAL_ABUNDANCE.iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
    to_value(&map).unwrap()
}
