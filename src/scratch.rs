//! Scratch — create and manage persistent scratch buffers.
//!
//! Scratch buffers are stored in `$XDG_DATA_HOME/nvim/scratch/` (or
//! `~/.local/share/nvim/scratch/` as fallback). Each scratch buffer is a
//! normal file that persists across sessions.

use std::path::{Path, PathBuf};

use nvim_oxi::api;
use tane::prelude::*;

/// Return the scratch directory path.
///
/// Uses `$XDG_DATA_HOME/nvim/scratch/` if `XDG_DATA_HOME` is set,
/// otherwise falls back to `~/.local/share/nvim/scratch/`.
#[must_use]
pub fn scratch_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        Path::new(&xdg).join("nvim").join("scratch")
    } else if let Ok(home) = std::env::var("HOME") {
        Path::new(&home)
            .join(".local")
            .join("share")
            .join("nvim")
            .join("scratch")
    } else {
        PathBuf::from("/tmp/nvim-scratch")
    }
}

/// Generate a default scratch file name based on timestamp.
#[must_use]
pub fn default_scratch_name() -> String {
    use std::time::SystemTime;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("scratch-{ts}.md")
}

/// Sanitize a user-provided scratch name for use as a filename.
///
/// Replaces path separators and other problematic characters with dashes.
#[must_use]
pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '-' })
        .collect()
}

/// Build the full path for a scratch file.
#[must_use]
pub fn scratch_path(name: Option<&str>) -> PathBuf {
    let dir = scratch_dir();
    let filename = match name {
        Some(n) => sanitize_name(n),
        None => default_scratch_name(),
    };
    dir.join(filename)
}

/// Ensure the scratch directory exists.
fn ensure_scratch_dir() -> Result<(), std::io::Error> {
    let dir = scratch_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Open or create a scratch buffer.
///
/// If `name` is `None`, a timestamped name is generated. The file is created
/// in the scratch directory and opened in the current window.
pub fn open_scratch(name: Option<&str>) -> Result<(), oxi::Error> {
    ensure_scratch_dir().map_err(|e| {
        oxi::Error::Api(oxi::api::Error::Other(format!(
            "Failed to create scratch directory: {e}"
        )))
    })?;

    let path = scratch_path(name);

    // Create the file if it doesn't exist
    if !path.exists() {
        std::fs::File::create(&path).map_err(|e| {
            oxi::Error::Api(oxi::api::Error::Other(format!(
                "Failed to create scratch file: {e}"
            )))
        })?;
    }

    let path_str = path.to_string_lossy();
    api::command(&format!("edit {path_str}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Safety helper: save, set, and restore an env var around a test body.
    /// The closure receives the set env context and can call `scratch_dir()`,
    /// etc. while the env var is overridden.
    ///
    /// # Safety
    /// `set_var`/`remove_var` are unsafe in edition 2024 because modifying
    /// environment variables is not thread-safe. Tests using this helper should
    /// run with `--test-threads=1` or accept the risk of concurrent env
    /// mutation in test suites.
    unsafe fn with_xdg<F: FnOnce()>(value: &str, body: F) {
        let original = std::env::var("XDG_DATA_HOME").ok();
        unsafe { std::env::set_var("XDG_DATA_HOME", value) };
        body();
        match original {
            Some(val) => unsafe { std::env::set_var("XDG_DATA_HOME", val) },
            None => unsafe { std::env::remove_var("XDG_DATA_HOME") },
        }
    }

    #[test]
    fn scratch_dir_uses_xdg_data_home() {
        unsafe {
            with_xdg("/tmp/test-xdg", || {
                let dir = scratch_dir();
                assert_eq!(dir, PathBuf::from("/tmp/test-xdg/nvim/scratch"));
            });
        }
    }

    #[test]
    fn sanitize_name_replaces_slashes() {
        assert_eq!(sanitize_name("foo/bar"), "foo-bar");
        assert_eq!(sanitize_name("hello world.txt"), "hello-world.txt");
    }

    #[test]
    fn sanitize_name_preserves_valid_chars() {
        assert_eq!(sanitize_name("my-file_2.md"), "my-file_2.md");
    }

    #[test]
    fn sanitize_name_handles_empty_string() {
        assert_eq!(sanitize_name(""), "");
    }

    #[test]
    fn sanitize_name_handles_special_chars() {
        assert_eq!(sanitize_name("a@b#c$d"), "a-b-c-d");
    }

    #[test]
    fn default_scratch_name_has_md_extension() {
        let name = default_scratch_name();
        assert!(name.starts_with("scratch-"));
        assert!(name.ends_with(".md"));
    }

    #[test]
    fn default_scratch_name_contains_timestamp() {
        let name = default_scratch_name();
        let ts_part = name
            .strip_prefix("scratch-")
            .unwrap()
            .strip_suffix(".md")
            .unwrap();
        assert!(ts_part.parse::<u64>().is_ok());
    }

    #[test]
    fn scratch_path_with_name() {
        unsafe {
            with_xdg("/tmp/test-xdg-sp", || {
                let path = scratch_path(Some("notes.md"));
                assert_eq!(
                    path,
                    PathBuf::from("/tmp/test-xdg-sp/nvim/scratch/notes.md")
                );
            });
        }
    }

    #[test]
    fn scratch_path_without_name_generates_default() {
        unsafe {
            with_xdg("/tmp/test-xdg-sp2", || {
                let path = scratch_path(None);
                let filename = path.file_name().unwrap().to_str().unwrap();
                assert!(filename.starts_with("scratch-"));
                assert!(filename.ends_with(".md"));
            });
        }
    }

    #[test]
    fn ensure_scratch_dir_creates_directory() {
        let temp = tempfile::tempdir().unwrap();
        let xdg_path = temp.path().join("xdg");
        unsafe {
            with_xdg(xdg_path.to_str().unwrap(), || {
                ensure_scratch_dir().unwrap();

                let expected = xdg_path.join("nvim").join("scratch");
                assert!(expected.exists());
                assert!(expected.is_dir());
            });
        }
    }

    #[test]
    fn ensure_scratch_dir_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let xdg_path = temp.path().join("xdg2");
        unsafe {
            with_xdg(xdg_path.to_str().unwrap(), || {
                ensure_scratch_dir().unwrap();
                ensure_scratch_dir().unwrap(); // Should not error
            });
        }
    }
}
