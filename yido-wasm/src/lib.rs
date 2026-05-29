use wasm_bindgen::prelude::*;
use yido_core::{Composer, Layout};

#[wasm_bindgen]
pub struct YidoEngine {
    composer: Composer,
}

#[wasm_bindgen]
impl YidoEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(layout_toml: &str) -> Result<YidoEngine, JsValue> {
        let layout = Layout::from_toml(layout_toml)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self {
            composer: Composer::new(layout),
        })
    }

    #[wasm_bindgen(js_name = inputKey)]
    pub fn input_key(&mut self, key: &str, shift: bool) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.input_key(key, shift))
    }

    pub fn backspace(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.backspace())
    }

    pub fn flush(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.flush())
    }

    pub fn cancel(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.cancel())
    }
}

fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}
