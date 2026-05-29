use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crate_name() -> String {
    yido_core::crate_name().to_owned()
}
