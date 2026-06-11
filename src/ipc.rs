//! wasm-bindgen glue for the Tauri IPC bridge (window.__TAURI__).

use serde_json::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    async fn tauri_listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

fn to_js(v: &Value) -> JsValue {
    serde_wasm_bindgen::to_value(v).unwrap_or(JsValue::NULL)
}

fn from_js(v: JsValue) -> Value {
    serde_wasm_bindgen::from_value(v).unwrap_or(Value::Null)
}

/// Invoke a Tauri command with JSON args; both ok and err arrive as JSON.
pub async fn invoke(cmd: &str, args: Value) -> Result<Value, Value> {
    match tauri_invoke(cmd, to_js(&args)).await {
        Ok(v) => Ok(from_js(v)),
        Err(e) => Err(from_js(e)),
    }
}

/// Subscribe to a Tauri event for the lifetime of the app.
pub fn listen_forever(event: &'static str, mut handler: impl FnMut(Value) + 'static) {
    wasm_bindgen_futures::spawn_local(async move {
        let closure = Closure::wrap(Box::new(move |ev: JsValue| {
            let v = from_js(ev);
            let payload = v.get("payload").cloned().unwrap_or(Value::Null);
            handler(payload);
        }) as Box<dyn FnMut(JsValue)>);
        tauri_listen(event, closure.as_ref().unchecked_ref()).await;
        closure.forget();
    });
}
