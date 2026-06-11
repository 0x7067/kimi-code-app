//! F-010: embedded-terminal commands. PTY output streams to the frontend as
//! `term:output` events (lossy UTF-8); shell exit emits `term:exit`.

use crate::terminal::{spawn_shell, Registry, Term};
use serde_json::{json, Value};
use std::io::Read;
use std::io::Write;
use tauri::{AppHandle, Emitter, State};

/// Tauri-managed registry of open terminals.
#[derive(Default)]
pub struct TermState(pub std::sync::Mutex<Registry<Term>>);

fn lock(state: &State<'_, TermState>) -> Result<std::sync::MutexGuard<'_, Registry<Term>>, String> {
    state.0.lock().map_err(|_| "terminal registry poisoned".to_string())
}

/// Open a new terminal running the user's shell in `cwd` (the selected
/// project). Returns `{ "id": n }`; output arrives via `term:output` events.
#[tauri::command]
pub fn term_open(
    app: AppHandle,
    state: State<'_, TermState>,
    cwd: Option<String>,
    rows: Option<u16>,
    cols: Option<u16>,
) -> Result<Value, String> {
    let (term, mut reader) = spawn_shell(cwd.as_deref(), rows.unwrap_or(24), cols.unwrap_or(120))?;
    let id = lock(&state)?.insert(term);
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app.emit("term:output", json!({"id": id, "data": data}));
                }
            }
        }
        let _ = app.emit("term:exit", json!({"id": id}));
    });
    Ok(json!({"id": id}))
}

/// Write user input to the terminal's stdin.
#[tauri::command]
pub fn term_write(state: State<'_, TermState>, id: u64, data: String) -> Result<(), String> {
    let mut reg = lock(&state)?;
    let term = reg.get_mut(id).ok_or("no such terminal")?;
    term.writer.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    term.writer.flush().map_err(|e| e.to_string())
}

/// Resize the PTY.
#[tauri::command]
pub fn term_resize(state: State<'_, TermState>, id: u64, rows: u16, cols: u16) -> Result<(), String> {
    let mut reg = lock(&state)?;
    reg.get_mut(id).ok_or("no such terminal")?.resize(rows, cols)
}

/// Close a terminal, killing its shell.
#[tauri::command]
pub fn term_close(state: State<'_, TermState>, id: u64) -> Result<(), String> {
    let term = lock(&state)?.remove(id);
    if let Some(mut term) = term {
        let _ = term.child.kill();
    }
    Ok(())
}
