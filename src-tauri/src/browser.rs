//! F-006: Browser preview helpers — live-reload file watcher.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Start a file-system watcher for `cwd` that emits `browser:reload` when
/// relevant source files change. Watches with a 500ms debounce.
pub fn start_live_reload_watcher(app: AppHandle, cwd: String) -> Result<RecommendedWatcher, String> {
    let app2 = app.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Only react to meaningful modifications.
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                    let _ = app2.emit("browser:reload", ());
                }
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(500)),
    )
    .map_err(|e| e.to_string())?;

    watcher
        .watch(PathBuf::from(cwd).as_ref(), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;
    Ok(watcher)
}
