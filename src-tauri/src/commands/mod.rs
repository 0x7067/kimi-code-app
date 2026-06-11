//! Tauri commands, grouped by concern. All are registered in `lib.rs`.

mod acp;
mod config;
mod dialogs;
mod git;
mod kimi;
mod projects;
mod sessions;

pub use acp::AppState;
pub use sessions::spawn_session_index_watcher;

/// Generate the Tauri invoke handler with all commands.
/// Kept inside this module so the macro can see the private submodules.
pub fn handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Clone + Send + Sync + 'static {
    tauri::generate_handler![
        acp::acp_connect,
        acp::acp_request,
        acp::acp_notify,
        acp::acp_cancel,
        acp::acp_steer,
        acp::acp_respond_permission,
        sessions::kimi_list_sessions,
        sessions::kimi_load_session,
        kimi::kimi_login,
        kimi::js_log,
        kimi::kimi_version,
        config::read_kimi_config,
        config::write_kimi_config,
        projects::recent_projects,
        dialogs::pick_folder,
        dialogs::pick_image,
        projects::mcp_servers,
        git::git_diff,
    ]
}
