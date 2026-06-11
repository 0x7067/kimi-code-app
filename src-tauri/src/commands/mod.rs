//! Tauri commands, grouped by concern. All are registered in `lib.rs`.

mod acp;
mod config;
mod dialogs;
mod git;
mod kimi;
mod projects;

pub use acp::AppState;

/// Generate the Tauri invoke handler with all commands.
/// Kept inside this module so the macro can see the private submodules.
pub fn handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Clone + Send + Sync + 'static {
    tauri::generate_handler![
        acp::acp_connect,
        acp::acp_request,
        acp::acp_notify,
        acp::acp_respond_permission,
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
