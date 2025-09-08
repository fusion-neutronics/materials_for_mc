#![cfg(all(target_arch = "wasm32", feature = "wasm"))]
use materials_for_mc::material_wasm::WasmMaterial;
use wasm_bindgen_test::*;

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
    // Minimal valid JSON for a nuclide (must be valid JSON; raw string should not escape quotes)
    let nuclide_name = "Li6";
    let json_content = r#"{ "name": "Li6" }"#; // Parser accepts nuclide with just a name
    let result = material.load_nuclide_data(nuclide_name, json_content);
    assert!(result.is_ok(), "Failed to load nuclide data from JSON");
    // Add nuclide after loading data
    material
        .add_nuclide(nuclide_name, 1.0)
        .expect("Failed to add nuclide");
    let nuclides = material.get_nuclides();
    assert!(
        nuclides
            .iter()
            .any(|n| n.as_string().unwrap() == nuclide_name),
        "Nuclide not found after loading JSON"
    );
}

#[wasm_bindgen_test]
fn test_load_nuclear_data_from_json_file() {
    let mut material = WasmMaterial::new();
    let nuclide_name = "Li6";
    // Load the JSON content from the file at compile time
    let json_content = include_str!("../tests/Li6.json");
    material
        .add_nuclide(nuclide_name, 1.0)
        .expect("Failed to add nuclide");
    let result = material.load_nuclide_data(nuclide_name, json_content);
    assert!(result.is_ok(), "Failed to load nuclide data from JSON file");
    let nuclides = material.get_nuclides();
    assert!(
        nuclides
            .iter()
            .any(|n| n.as_string().unwrap() == nuclide_name),
        "Nuclide not found after loading JSON file"
    );
}

#[wasm_bindgen_test]
fn test_reaction_mts_after_loading_json_file() {
    let mut material = WasmMaterial::new();
    let nuclide_name = "Li6";
    let json_content = include_str!("../tests/Li6.json");
    material
        .add_nuclide(nuclide_name, 1.0)
        .expect("Failed to add nuclide");
    let result = material.load_nuclide_data(nuclide_name, json_content);
    assert!(result.is_ok(), "Failed to load nuclide data from JSON file");

    // Force load via ensure_nuclides_loaded to populate inner.nuclide_data
    material
        .ensure_nuclides_loaded()
        .expect("ensure_nuclides_loaded failed");

    // Attempt to get reaction MTs
    let mts_js = material.reaction_mts().expect("reaction_mts() failed");
    let mts: Vec<i32> = mts_js
        .iter()
        .filter_map(|v| v.as_f64().map(|f| f as i32))
        .collect();
    assert!(
        !mts.is_empty(),
        "Reaction MTs array should not be empty after loading nuclide"
    );
    // Li6 test JSON should contain at least MT 2 or MT 1 (total)
    assert!(
        mts.contains(&2) || mts.contains(&1),
        "Expected common MT (1 or 2) to be present"
    );
}
