//! Tauri commands, grouped by concern. All are registered in `lib.rs`.

mod acp;
mod automation;
mod checkpoint;
mod config;
mod dialogs;
mod git;
mod kimi;
mod memory;
mod mcp;
mod projects;
mod sessions;
mod settings_store;
pub mod terminal;

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
        sessions::kimi_session_activity,
        kimi::kimi_login,
        kimi::js_log,
        kimi::kimi_version,
        kimi::detect_kimi_binary,
        kimi::kimi_auth_status,
        config::read_kimi_config,
        config::write_kimi_config,
        config::list_kimi_models,
        config::set_default_model,
        settings_store::read_app_settings,
        settings_store::write_app_settings,
        dialogs::pick_file,
        projects::recent_projects,
        projects::list_files,
        dialogs::pick_folder,
        dialogs::pick_image,
        projects::mcp_servers,
        projects::read_agents_md,
        projects::index_project,
        git::git_diff,
        git::list_worktrees,
        git::create_worktree,
        git::remove_worktree,
        mcp::list_mcp_servers,
        mcp::save_mcp_server,
        mcp::delete_mcp_server,
        terminal::term_open,
        terminal::term_write,
        terminal::term_resize,
        terminal::term_close,
        checkpoint::save_checkpoint,
        checkpoint::list_checkpoints,
        checkpoint::load_checkpoint,
        checkpoint::delete_checkpoint,
        memory::list_memories,
        memory::save_memory,
        memory::delete_memory,
        memory::pin_memory,
        memory::retrieve_memories,
        memory::build_memory_context,
        automation::list_automation_runs,
        automation::run_automation_now,
    ]
}
