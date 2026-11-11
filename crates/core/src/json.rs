// SPDX-License-Identifier: MIT OR Apache-2.0

//! JSON text, read and written the way each build has it cheapest.
//!
//! A saved circuit is JSON (C-04), and reading and writing it is all this
//! crate needed JSON for. `serde_json` did it, and was a sixth of the wasm
//! bundle's code — the largest single thing in it, on the cold-load path M-21
//! budgets three seconds for, to do what every browser already does. So in
//! the browser the text is parsed and printed by the browser's own `JSON`, and
//! crosses the boundary as a JavaScript value through `serde-wasm-bindgen`,
//! which the control plane already ships (ADR 0002). Everywhere else — the
//! runner, the desktop's smoke test, every native test — `serde_json` does it
//! as before.
//!
//! The two agree on what they read. What they write differs only in how a
//! whole number is spelled: `serde_json` writes a 9 V battery as `9.0`, the
//! browser as `9`, and both read either.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Reads a value out of JSON text. The error is the parser's own words.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    imp::from_str(text)
}

/// Writes a value as compact JSON text.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, String> {
    imp::to_string(value)
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = JSON, js_name = parse, catch)]
        fn parse(text: &str) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(js_namespace = JSON, js_name = stringify, catch)]
        fn stringify(value: &JsValue) -> Result<JsValue, JsValue>;

        /// What `JSON.parse` throws; only its message is wanted.
        type Thrown;

        #[wasm_bindgen(method, getter)]
        fn message(this: &Thrown) -> Option<String>;
    }

    fn said(error: JsValue) -> String {
        error
            .unchecked_into::<Thrown>()
            .message()
            .unwrap_or_else(|| "not JSON".to_string())
    }

    pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, String> {
        let value = parse(text).map_err(said)?;
        serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
    }

    pub fn to_string<T: Serialize>(value: &T) -> Result<String, String> {
        // As JSON would have it: maps as objects, nothing as `null`.
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        let value = value
            .serialize(&serializer)
            .map_err(|error| error.to_string())?;
        stringify(&value)
            .map_err(said)?
            .as_string()
            .ok_or_else(|| "nothing to write".to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, String> {
        serde_json::from_str(text).map_err(|error| error.to_string())
    }

    pub fn to_string<T: Serialize>(value: &T) -> Result<String, String> {
        serde_json::to_string(value).map_err(|error| error.to_string())
    }
}
