//! wasm-bindgen glue for the Tauri IPC bridge (window.__TAURI__).
//!
//! The bridge is looked up dynamically via `Reflect` instead of
//! `#[wasm_bindgen(js_namespace = ...)]` imports. With static imports, a
//! missing `window.__TAURI__` (e.g. when the frontend runs in a plain
//! browser) throws synchronously inside the generated shim; even with
//! `catch`, the async-import machinery then calls `.then` on `undefined`
//! through the uncatchable js-sys `Promise::then` import, which aborts wasm
//! mid-poll and poisons the wasm-bindgen-futures executor — every later
//! task wake panics with "RefCell already borrowed".

use js_sys::{Function, Promise, Reflect};
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

fn to_js(v: &Value) -> JsValue {
    serde_wasm_bindgen::to_value(v).unwrap_or(JsValue::NULL)
}

fn from_js(v: JsValue) -> Value {
    serde_wasm_bindgen::from_value(v).unwrap_or(Value::Null)
}

/// Resolve `window.__TAURI__.<ns>.<name>` without ever throwing.
/// Returns the namespace object (the `this` for the call) and the function.
fn tauri_api(ns: &str, name: &str) -> Result<(JsValue, Function), Value> {
    let missing = || Value::String(format!("Tauri bridge unavailable (window.__TAURI__.{ns}.{name})"));
    let tauri = Reflect::get(&js_sys::global(), &"__TAURI__".into()).map_err(|_| missing())?;
    let ns_obj = Reflect::get(&tauri, &ns.into()).map_err(|_| missing())?;
    let f = Reflect::get(&ns_obj, &name.into())
        .ok()
        .and_then(|f| f.dyn_into::<Function>().ok())
        .ok_or_else(missing)?;
    Ok((ns_obj, f))
}

/// Call a bridge function and await the Promise it returns.
async fn call_async(ns: &str, name: &str, args: &[JsValue]) -> Result<Value, Value> {
    let (this, f) = tauri_api(ns, name)?;
    let js_args = js_sys::Array::new();
    for a in args {
        js_args.push(a);
    }
    let ret = Reflect::apply(&f, &this, &js_args).map_err(from_js)?;
    let promise: Promise = ret
        .dyn_into()
        .map_err(|_| Value::String(format!("window.__TAURI__.{ns}.{name} did not return a Promise")))?;
    match JsFuture::from(promise).await {
        Ok(v) => Ok(from_js(v)),
        Err(e) => Err(from_js(e)),
    }
}

/// Invoke a Tauri command with JSON args; both ok and err arrive as JSON.
pub async fn invoke(cmd: &str, args: Value) -> Result<Value, Value> {
    call_async("core", "invoke", &[JsValue::from_str(cmd), to_js(&args)]).await
}

/// Subscribe to a Tauri event for the lifetime of the app.
pub fn listen_forever(event: &'static str, mut handler: impl FnMut(Value) + 'static) {
    wasm_bindgen_futures::spawn_local(async move {
        let closure = Closure::wrap(Box::new(move |ev: JsValue| {
            let v = from_js(ev);
            let payload = v.get("payload").cloned().unwrap_or(Value::Null);
            handler(payload);
        }) as Box<dyn FnMut(JsValue)>);
        let cb: JsValue = closure.as_ref().clone();
        match call_async("event", "listen", &[JsValue::from_str(event), cb]).await {
            Ok(_) => closure.forget(),
            Err(e) => {
                // No Tauri bridge (plain browser) or listen failed: drop the
                // closure and keep the app alive without this event stream.
                web_sys::console::warn_1(&format!("tauri listen({event}) unavailable: {e}").into());
            }
        }
    });
}
