mod acp;
mod commands;

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
        .invoke_handler(tauri::generate_handler![
            commands::acp_connect,
            commands::acp_request,
            commands::acp_notify,
            commands::acp_respond_permission,
            commands::kimi_login,
            commands::js_log,
            commands::kimi_version,
            commands::read_kimi_config,
            commands::write_kimi_config,
            commands::recent_projects,
            commands::pick_folder,
            commands::pick_image,
            commands::mcp_servers,
            commands::git_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
