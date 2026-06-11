mod acp;
mod commands;
mod paths;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .on_page_load(|window, _| {
            let _ = window.eval(
                "window.addEventListener('error', e => window.__TAURI__.core.invoke('js_log', {msg: String(e.message)+' @ '+String(e.filename)+':'+String(e.lineno)}));\
                 window.addEventListener('unhandledrejection', e => window.__TAURI__.core.invoke('js_log', {msg: 'rejection: '+String(e.reason)}));",
            );
        })
        .invoke_handler(commands::handlers())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
