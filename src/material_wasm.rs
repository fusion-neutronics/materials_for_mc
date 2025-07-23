use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::material::Material;
use js_sys::{Array, Map, JSON};

#[wasm_bindgen]
pub struct WasmMaterial {
    inner: Material,
}

#[derive(Serialize, Deserialize)]
struct MacroscopicXsResult {
    energy_grid: Vec<f64>,
    cross_sections: HashMap<String, Vec<f64>>,
}

#[wasm_bindgen]
impl WasmMaterial {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmMaterial {
            inner: Material::new(),
        }
    }

    #[wasm_bindgen]
    pub fn add_nuclide(&mut self, nuclide: &str, fraction: f64) -> Result<(), JsValue> {
        self.inner.add_nuclide(nuclide, fraction)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn add_element(&mut self, element: &str, fraction: f64) -> Result<(), JsValue> {
        self.inner
            .add_element(element, fraction)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn set_density(&mut self, unit: &str, value: f64) -> Result<(), JsValue> {
        self.inner.set_density(unit, value)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn set_volume(&mut self, value: f64) -> Result<(), JsValue> {
        self.inner.volume(Some(value))
            .map_err(|e| JsValue::from_str(&e))
            .map(|_| ())
    }

    #[wasm_bindgen]
    pub fn set_temperature(&mut self, temperature: &str) {
        self.inner.set_temperature(temperature);
    }

    #[wasm_bindgen]
    pub fn get_nuclides(&self) -> Array {
        let nuclides = self.inner.get_nuclides();
        nuclides.into_iter()
            .map(|n| JsValue::from_str(&n))
            .collect::<Array>()
    }

    #[wasm_bindgen]
    pub fn get_atoms_per_cc(&self) -> Result<JsValue, JsValue> {
        // Safe to use try-catch pattern with WASM since panics will be converted to JS exceptions
        if self.inner.density.is_none() {
            return Err(JsValue::from_str("Cannot calculate atoms per cc: Material has no density defined"));
        }
        
        if self.inner.nuclides.is_empty() {
            return Err(JsValue::from_str("Cannot calculate atoms per cc: Material has no nuclides defined"));
        }
        
        // Now it's safe to call get_atoms_per_cc without risk of panic
        let atoms_per_cc = self.inner.get_atoms_per_cc();
        
        let map = Map::new();
        for (nuclide, density) in atoms_per_cc {
            map.set(&JsValue::from_str(&nuclide), &JsValue::from_f64(density));
        }
        Ok(map.into())
    }

    #[wasm_bindgen]
    pub fn calculate_macroscopic_xs_neutron(&mut self, mt_filter: Option<Array>) -> Result<JsValue, JsValue> {
        // Check preconditions to avoid panics
        if self.inner.density.is_none() {
            return Err(JsValue::from_str("Cannot calculate macroscopic cross sections: Material has no density defined"));
        }
        if self.inner.nuclides.is_empty() {
            return Err(JsValue::from_str("Cannot calculate macroscopic cross sections: Material has no nuclides defined"));
        }
        // First ensure nuclides are loaded using our WASM-specific function
        if let Err(e) = self.ensure_nuclides_loaded() {
            return Err(JsValue::from_str(&format!("{}", e)));
        }
        // Convert JS Array to Option<Vec<String>>
        let mt_filter_vec = mt_filter.map(|arr| {
            arr.iter().filter_map(|v| v.as_string()).collect::<Vec<String>>()
        });
        let xs = self.inner.calculate_macroscopic_xs_neutron(None, mt_filter_vec.as_ref());
        let energy_grid = self.inner.unified_energy_grid_neutron.clone();
        let data = MacroscopicXsResult {
            energy_grid,
            cross_sections: xs,
        };
        let serialized = serde_json::to_string(&data)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        Ok(JSON::parse(&serialized)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?)
    }

    #[wasm_bindgen]
    pub fn calculate_total_xs_neutron(&mut self) -> Result<JsValue, JsValue> {
        // Check preconditions to avoid panics
        if self.inner.density.is_none() {
            return Err(JsValue::from_str("Cannot calculate total cross sections: Material has no density defined"));
        }
        
        if self.inner.nuclides.is_empty() {
            return Err(JsValue::from_str("Cannot calculate total cross sections: Material has no nuclides defined"));
        }
        
        // First ensure nuclides are loaded using our WASM-specific function
        if let Err(e) = self.ensure_nuclides_loaded() {
            return Err(JsValue::from_str(&format!("{}", e)));
        }
        
        // Now it's safer to call the methods
        let xs = self.inner.calculate_total_xs_neutron();
        let energy_grid = self.inner.unified_energy_grid_neutron.clone();
        
        let data = MacroscopicXsResult {
            energy_grid,
            cross_sections: xs,
        };
        
        let serialized = serde_json::to_string(&data)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        
        Ok(JSON::parse(&serialized)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?)
    }

    #[wasm_bindgen]
    pub fn reaction_mts(&mut self) -> Result<Array, JsValue> {
        // Ensure nuclides are loaded and get the sorted MT list
        let mts = self.inner.reaction_mts()
            .map_err(|e| JsValue::from_str(&format!("Failed to get MT numbers: {}", e)))?;
        Ok(mts.into_iter().map(JsValue::from).collect::<Array>())
    }

    #[wasm_bindgen]
    pub fn mean_free_path_neutron(&mut self, energy: f64) -> Option<f64> {
        self.inner.mean_free_path_neutron(energy)
    }

    #[wasm_bindgen]
    pub fn load_nuclide_data(&mut self, nuclide_name: &str, json_content: &str) -> Result<(), JsValue> {
        match crate::nuclide_wasm::set_nuclide_json_content(nuclide_name, json_content) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsValue::from_str(&format!("Failed to load nuclide data: {:?}", e))),
        }
    }
    
    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        format!("{:?}", self.inner)
    }
}

// Override the ensure_nuclides_loaded method to use WASM-specific functions
impl WasmMaterial {
    // Override ensure_nuclides_loaded to use WASM-specific version
    pub fn ensure_nuclides_loaded(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let nuclide_names: Vec<String> = self.inner.nuclides.keys()
            .filter(|name| !self.inner.nuclide_data.contains_key(*name))
            .cloned()
            .collect();
        
        if nuclide_names.is_empty() {
            return Ok(());
        }
        
        // Get the global configuration
        let config = crate::config::CONFIG.lock().unwrap();
        
        // Load any missing nuclides using the WASM-specific function
        for nuclide_name in nuclide_names {
            match crate::nuclide_wasm::get_or_load_nuclide_wasm(&nuclide_name, &config.cross_sections) {
                Ok(nuclide) => {
                    self.inner.nuclide_data.insert(nuclide_name.clone(), nuclide);
                },
                Err(_) => {
                    // In WASM, provide a more specific error message
                    return Err(format!("Failed to load nuclide '{}' in WASM environment. Make sure you've loaded the nuclide data first using WasmConfig.set_nuclide_data() or WasmMaterial.load_nuclide_data()", nuclide_name).into());
                }
            }
        }
        
        Ok(())
    }
}