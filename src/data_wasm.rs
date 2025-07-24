use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use crate::data::{NATURAL_ABUNDANCE, ELEMENT_NUCLIDES, SUM_RULES, ELEMENT_NAMES, ATOMIC_MASSES};

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

#[wasm_bindgen]
pub fn sum_rules() -> JsValue {
    let map: std::collections::HashMap<i32, Vec<i32>> = SUM_RULES.iter().map(|(k, v)| (*k, v.clone())).collect();
    to_value(&map).unwrap()
}

#[wasm_bindgen]
pub fn element_names() -> JsValue {
    let map: std::collections::HashMap<String, String> = ELEMENT_NAMES.iter().map(|(symbol, name)| ((*symbol).to_string(), (*name).to_string())).collect();
    to_value(&map).unwrap()
}

#[wasm_bindgen]
pub fn atomic_masses() -> JsValue {
    let map: std::collections::HashMap<String, f64> = ATOMIC_MASSES.iter().map(|(nuclide, mass)| ((*nuclide).to_string(), *mass)).collect();
    to_value(&map).unwrap()
}

#[wasm_bindgen]
pub fn wasm_get_all_mt_descendants(mt_num: i32) -> js_sys::Array {
    let sum_rules = &crate::data::SUM_RULES;
    let mut out = std::collections::HashSet::new();
    get_all_mt_descendants(mt_num, sum_rules, &mut out);
    let mut v: Vec<i32> = out.into_iter().collect();
    v.sort();
    v.into_iter().map(|x| js_sys::Number::from(x).into()).collect()
}