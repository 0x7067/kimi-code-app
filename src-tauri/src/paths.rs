//! Locations of the kimi CLI and its config home.

use std::path::PathBuf;

/// Locate the kimi binary. GUI apps launched from Finder get a minimal PATH,
/// so check the known install locations before falling back to PATH lookup.
pub fn kimi_bin() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let candidates = [
        kimi_home().join("bin/kimi"),
        home.join(".local/bin/kimi"),
        "/usr/local/bin/kimi".into(),
        "/opt/homebrew/bin/kimi".into(),
    ];
    for c in candidates {
        if c.is_file() {
            return c;
        }
    }
    "kimi".into()
}

pub fn kimi_home() -> PathBuf {
    std::env::var("KIMI_CODE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".kimi-code"))
}
