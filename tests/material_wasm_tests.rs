use wasm_bindgen_test::*;
use materials_for_mc::material_wasm::WasmMaterial;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_wasm_material() {
    let material = WasmMaterial::new();
    // Check that the material is created and has no nuclides by default
    assert_eq!(material.get_nuclides().length(), 0);
}
