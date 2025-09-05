use wasm_bindgen::prelude::*;
use crate::element::Element;

#[wasm_bindgen]
pub struct WasmElement {
	inner: Element,
}

#[wasm_bindgen]
impl WasmElement {
	#[wasm_bindgen(constructor)]
	pub fn new(name: String) -> WasmElement { WasmElement { inner: Element::new(name) } }

	#[wasm_bindgen(getter)]
	pub fn name(&self) -> String { self.inner.name.clone() }

	#[wasm_bindgen(js_name = getElementIsotopes)]
	pub fn get_element_isotopes(&self) -> js_sys::Array {
		self.inner.get_element_isotopes().into_iter().map(JsValue::from).collect()
	}
}
