use wasm_bindgen_test::*;
use materials_for_mc::material_wasm::WasmMaterial;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_wasm_material() {
    let material = WasmMaterial::new();
    // Check that the material is created and has no nuclides by default
    assert_eq!(material.get_nuclides().length(), 0);
}

#[wasm_bindgen_test]
fn test_load_nuclear_data_from_json() {
    let mut material = WasmMaterial::new();
    // Example minimal JSON for a nuclide (adjust fields as needed for your project)
    let nuclide_name = "Li6";
    let json_content = r#"{
        \"name\": \"Li6\",
        \"atomic_number\": 3,
        \"mass_number\": 6,
        \"cross_sections\": { \"1\": [0.1, 0.2, 0.3] }
    }"#;
    let result = material.load_nuclide_data(nuclide_name, json_content);
    assert!(result.is_ok(), "Failed to load nuclide data from JSON");
    // Optionally, check that the nuclide is now present
    material.add_nuclide(nuclide_name, 1.0).expect("Failed to add nuclide");
    let nuclides = material.get_nuclides();
    assert!(nuclides.iter().any(|n| n.as_string().unwrap() == nuclide_name), "Nuclide not found after loading JSON");
}

#[wasm_bindgen_test]
fn test_load_nuclear_data_from_json_file() {
    let mut material = WasmMaterial::new();
    let nuclide_name = "Li6";
    // Load the JSON content from the file at compile time
    let json_content = include_str!("../tests/Li6.json");
    material.add_nuclide(nuclide_name, 1.0).expect("Failed to add nuclide");
    let result = material.load_nuclide_data(nuclide_name, json_content);
    assert!(result.is_ok(), "Failed to load nuclide data from JSON file");
    let nuclides = material.get_nuclides();
    assert!(nuclides.iter().any(|n| n.as_string().unwrap() == nuclide_name), "Nuclide not found after loading JSON file");
}

#[wasm_bindgen_test]
fn test_reaction_mts_after_loading_json_file() {
    let mut material = WasmMaterial::new();
    let nuclide_name = "Li6";
    let json_content = include_str!("../tests/Li6.json");
    material.add_nuclide(nuclide_name, 1.0).expect("Failed to add nuclide");
    let result = material.load_nuclide_data(nuclide_name, json_content);
    assert!(result.is_ok(), "Failed to load nuclide data from JSON file");

    // Force load via ensure_nuclides_loaded to populate inner.nuclide_data
    material.ensure_nuclides_loaded().expect("ensure_nuclides_loaded failed");

    // Debug: print nuclide_data after loading
    let nuclide_data_str = material.debug_nuclide_data();
    web_sys::console::log_1(&format!("nuclide_data after load: {}", nuclide_data_str).into());

    // Attempt to get reaction MTs
    let mts_js = material.reaction_mts().expect("reaction_mts() failed");
    let mts: Vec<i32> = mts_js.iter().filter_map(|v| v.as_f64().map(|f| f as i32)).collect();
    web_sys::console::log_1(&format!("mts: {:?}", mts).into());
    assert!(!mts.is_empty(), "Reaction MTs array should not be empty after loading nuclide");
    // Li6 test JSON should contain at least MT 2 (elastic) or other standard reactions; relax expectation if MT 1 not present
    assert!(mts.contains(&2) || mts.contains(&1), "Expected common MT (1 or 2) to be present");
}
