use wasm_bindgen::prelude::*;
use crate::data::NATURAL_ABUNDANCE;

#[wasm_bindgen]
pub fn natural_abundance() -> JsValue {
    let map: std::collections::HashMap<String, f64> = NATURAL_ABUNDANCE.iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
    JsValue::from_serde(&map).unwrap()
}
