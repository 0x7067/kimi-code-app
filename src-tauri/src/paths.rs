//! Locations of the kimi CLI and its config home.

use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// User-configured kimi binary override (F-011.1), applied from app settings
/// at startup and whenever settings are saved.
static KIMI_OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Set (or clear) the configured kimi binary override.
pub fn set_kimi_override(path: Option<String>) {
    let value = path.filter(|p| !p.trim().is_empty()).map(PathBuf::from);
    if let Ok(mut guard) = KIMI_OVERRIDE.write() {
        *guard = value;
    }
}

pub fn kimi_override() -> Option<PathBuf> {
    KIMI_OVERRIDE.read().ok().and_then(|g| g.clone())
}

/// Detection order for the kimi binary (F-011.1): configured override first,
/// then each directory on `PATH`, then the known install locations. Pure so
/// the ordering is unit-testable.
pub fn kimi_candidates(
    override_path: Option<&Path>,
    path_var: Option<&str>,
    kimi_home: &Path,
    home: &Path,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = override_path {
        out.push(p.to_path_buf());
    }
    if let Some(path) = path_var {
        for dir in std::env::split_paths(path) {
            if !dir.as_os_str().is_empty() {
                out.push(dir.join("kimi"));
            }
        }
    }
    for known in [
        kimi_home.join("bin/kimi"),
        home.join(".local/bin/kimi"),
        PathBuf::from("/usr/local/bin/kimi"),
        PathBuf::from("/opt/homebrew/bin/kimi"),
    ] {
        if !out.contains(&known) {
            out.push(known);
        }
    }
    out
}

/// Locate the kimi binary. GUI apps launched from Finder get a minimal PATH,
/// so the configured override and known install locations are checked first.
pub fn kimi_bin() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let path_var = std::env::var("PATH").ok();
    let kimi_home = kimi_home();
    for c in kimi_candidates(kimi_override().as_deref(), path_var.as_deref(), &kimi_home, &home) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> PathBuf {
        PathBuf::from("/Users/t")
    }

    fn khome() -> PathBuf {
        home().join(".kimi-code")
    }

    #[test]
    fn override_comes_first() {
        let cands = kimi_candidates(
            Some(Path::new("/custom/kimi")),
            Some("/usr/bin"),
            &khome(),
            &home(),
        );
        assert_eq!(cands[0], PathBuf::from("/custom/kimi"));
        assert_eq!(cands[1], PathBuf::from("/usr/bin/kimi"));
    }

    #[test]
    fn path_dirs_precede_known_locations_in_order() {
        let cands = kimi_candidates(None, Some("/a:/b"), &khome(), &home());
        assert_eq!(
            cands,
            vec![
                PathBuf::from("/a/kimi"),
                PathBuf::from("/b/kimi"),
                khome().join("bin/kimi"),
                home().join(".local/bin/kimi"),
                PathBuf::from("/usr/local/bin/kimi"),
                PathBuf::from("/opt/homebrew/bin/kimi"),
            ]
        );
    }

    #[test]
    fn known_locations_not_duplicated_when_on_path() {
        let cands = kimi_candidates(None, Some("/usr/local/bin"), &khome(), &home());
        let hits = cands
            .iter()
            .filter(|c| **c == PathBuf::from("/usr/local/bin/kimi"))
            .count();
        assert_eq!(hits, 1);
    }

    #[test]
    fn no_path_var_still_yields_known_locations() {
        let cands = kimi_candidates(None, None, &khome(), &home());
        assert_eq!(cands.len(), 4);
        assert_eq!(cands[0], khome().join("bin/kimi"));
    }
}
