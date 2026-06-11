//! F-010: PTY-backed embedded terminals.
//!
//! Each terminal is a `portable-pty` pair running the user's shell. The
//! generic `Registry` allocates ids (unit-tested without a real PTY); the
//! reader half is returned to the caller so command code can pump output
//! into Tauri events.

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};

/// One live terminal: stdin writer, master (for resize), and the child shell.
pub struct Term {
    pub writer: Box<dyn Write + Send>,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
}

/// Id-keyed registry with monotonically increasing ids (like sessions).
pub struct Registry<T> {
    next: u64,
    items: HashMap<u64, T>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self { next: 0, items: HashMap::new() }
    }
}

impl<T> Registry<T> {
    pub fn insert(&mut self, item: T) -> u64 {
        self.next += 1;
        self.items.insert(self.next, item);
        self.next
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    pub fn remove(&mut self, id: u64) -> Option<T> {
        self.items.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// The user's shell ($SHELL), falling back to /bin/sh.
pub fn default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "/bin/sh".into())
}

fn pty_size(rows: u16, cols: u16) -> PtySize {
    PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }
}

/// Spawn the user's shell on a fresh PTY. Environment is inherited from this
/// process and `cwd` (the selected project) becomes the working directory
/// (F-010.7). Returns the terminal handle plus the PTY output reader.
pub fn spawn_shell(
    cwd: Option<&str>,
    rows: u16,
    cols: u16,
) -> Result<(Term, Box<dyn Read + Send>), String> {
    let pty = native_pty_system();
    let pair = pty.openpty(pty_size(rows, cols)).map_err(|e| e.to_string())?;
    // CommandBuilder inherits the parent environment by default (F-010.7).
    let mut cmd = CommandBuilder::new(default_shell());
    if let Some(dir) = cwd.filter(|d| !d.trim().is_empty()) {
        cmd.cwd(dir);
    }
    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    drop(pair.slave);
    let reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    Ok((Term { writer, master: pair.master, child }, reader))
}

impl Term {
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        self.master.resize(pty_size(rows, cols)).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_allocates_increasing_ids_and_never_reuses() {
        let mut reg: Registry<&str> = Registry::default();
        let a = reg.insert("a");
        let b = reg.insert("b");
        assert!(b > a);
        assert_eq!(reg.remove(a), Some("a"));
        let c = reg.insert("c");
        assert!(c > b, "removed ids must not be reused");
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn registry_get_mut_and_missing_ids() {
        let mut reg: Registry<String> = Registry::default();
        let id = reg.insert("x".into());
        if let Some(v) = reg.get_mut(id) {
            v.push('y');
        }
        assert_eq!(reg.remove(id), Some("xy".into()));
        assert_eq!(reg.remove(id), None);
        assert!(reg.get_mut(999).is_none());
    }

    #[test]
    fn default_shell_is_never_empty() {
        assert!(!default_shell().is_empty());
    }
}
