use crate::config::CONFIG;
use js_sys::{Map, Object, Reflect};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmConfig;

#[wasm_bindgen]
impl WasmConfig {
    #[wasm_bindgen]
    pub fn set_cross_sections(js_map: &JsValue) -> Result<(), JsValue> {
        let obj = js_map.clone().dyn_into::<Object>()?;
        let mut cross_sections = HashMap::new();

        let keys = js_sys::Object::keys(&obj);
        let keys_len = keys.length();

        for i in 0..keys_len {
            let key = keys.get(i);
            let key_str = key
                .as_string()
                .ok_or_else(|| JsValue::from_str("Keys must be strings"))?;

            let value = Reflect::get(&obj, &key)?;
            let value_str = value
                .as_string()
                .ok_or_else(|| JsValue::from_str("Values must be strings"))?;

            cross_sections.insert(key_str, value_str);
        }

        // Access the global config and set the cross sections
        let mut config = CONFIG.lock().unwrap();
        config.set_cross_sections(cross_sections);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_cross_sections() -> Result<JsValue, JsValue> {
        // Access the global config and get the cross sections
        let config = CONFIG.lock().unwrap();
        let cross_sections = &config.cross_sections;

        let map = Map::new();

        for (key, value) in cross_sections {
            map.set(&JsValue::from_str(key), &JsValue::from_str(value));
        }

        Ok(map.into())
    }

    #[wasm_bindgen]
    pub fn set_nuclide_data(nuclide_name: &str, json_content: &str) -> Result<(), JsValue> {
        match crate::nuclide_wasm::set_nuclide_json_content(nuclide_name, json_content) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to set nuclide data: {:?}",
                e
            ))),
        }
    }
}
