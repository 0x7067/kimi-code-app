//! UI components, one file per view region.

mod app;
mod composer;
mod diff_pane;
mod login_modal;
mod permission_modal;
mod settings;
mod sidebar;
mod thread;
mod topbar;

pub use app::App;
pub(crate) use composer::Composer;
pub(crate) use diff_pane::DiffPane;
pub(crate) use login_modal::LoginModal;
pub(crate) use permission_modal::PermissionModal;
pub(crate) use settings::SettingsView;
pub(crate) use sidebar::Sidebar;
pub(crate) use thread::ThreadView;
pub(crate) use topbar::Topbar;
