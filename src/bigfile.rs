//! Bigfile — disable heavy features (treesitter, LSP, etc.) for large files.
//!
//! Installs a `BufReadPre` autocmd that checks file size against a configurable
//! threshold (default 1 MB). When a file exceeds the threshold, syntax
//! highlighting, filetype plugins, and other expensive features are disabled.

use nvim_oxi::api;
use nvim_oxi::api::opts::OptionOpts;
use tane::prelude::*;

/// Default file size threshold in bytes (1 MB).
pub const DEFAULT_THRESHOLD: u64 = 1_024 * 1_024;

/// Configuration for the bigfile module.
#[derive(Debug, Clone)]
pub struct BigfileConfig {
    /// File size threshold in bytes. Files larger than this get heavy features
    /// disabled.
    pub threshold: u64,
}

impl Default for BigfileConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

impl BigfileConfig {
    /// Read threshold from Neovim global variable `g:dougu_bigfile_threshold`.
    /// Falls back to [`DEFAULT_THRESHOLD`] if not set.
    pub fn from_vim() -> Self {
        let threshold = api::get_var::<i64>("dougu_bigfile_threshold")
            .ok()
            .and_then(|v| u64::try_from(v).ok())
            .unwrap_or(DEFAULT_THRESHOLD);
        Self { threshold }
    }
}

/// Check if a file exceeds the bigfile threshold.
#[must_use]
pub fn is_big_file(path: &str, threshold: u64) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() > threshold)
        .unwrap_or(false)
}

/// Disable heavy features on the current buffer.
///
/// This disables:
/// - Syntax highlighting (via `syntax off`)
/// - Filetype plugins (via `filetype off`)
/// - Swap file for this buffer
/// - Undo file for this buffer
/// - Fold method set to manual
fn disable_features_for_buffer() -> Result<(), oxi::Error> {
    let opts = OptionOpts::builder().scope(nvim_oxi::api::opts::OptionScope::Local).build();
    api::set_option_value("swapfile", false, &opts)?;
    api::set_option_value("undofile", false, &opts)?;
    api::set_option_value("foldmethod", "manual", &opts)?;
    api::set_option_value("undolevels", -1i64, &opts)?;

    // Disable syntax and filetype detection via ex commands
    api::command("syntax clear")?;
    api::command("filetype off")?;

    // Notify the user
    api::command("echomsg '[dougu] bigfile mode enabled — heavy features disabled'")?;
    Ok(())
}

/// Install the `BufReadPre` autocmd that triggers bigfile detection.
pub fn setup() -> Result<(), tane::Error> {
    Autocmd::on(&["BufReadPre"])
        .pattern("*")
        .group("dougu_bigfile")
        .desc("Dougu: disable heavy features for large files")
        .register(move |args| {
            let config = BigfileConfig::from_vim();
            let file = args.file;
            if let Some(path) = file.to_str() {
                if !path.is_empty() && is_big_file(path, config.threshold) {
                    disable_features_for_buffer().map_err(tane::Error::Oxi)?;
                }
            }
            Ok(false)
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_config_is_1mb() {
        let config = BigfileConfig::default();
        assert_eq!(config.threshold, 1_024 * 1_024);
    }

    #[test]
    fn custom_threshold() {
        let config = BigfileConfig { threshold: 512 };
        assert_eq!(config.threshold, 512);
    }

    #[test]
    fn is_big_file_returns_false_for_small_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("small.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"hello").unwrap();
        drop(f);

        assert!(!is_big_file(path.to_str().unwrap(), 1024));
    }

    #[test]
    fn is_big_file_returns_true_for_large_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        let data = vec![b'x'; 2048];
        f.write_all(&data).unwrap();
        drop(f);

        assert!(is_big_file(path.to_str().unwrap(), 1024));
    }

    #[test]
    fn is_big_file_returns_false_for_missing_file() {
        assert!(!is_big_file("/nonexistent/path/to/file.txt", 1024));
    }

    #[test]
    fn is_big_file_at_exact_threshold() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("exact.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        let data = vec![b'x'; 1024];
        f.write_all(&data).unwrap();
        drop(f);

        // Exactly at threshold should NOT be considered big (strictly greater).
        assert!(!is_big_file(path.to_str().unwrap(), 1024));
    }

    #[test]
    fn is_big_file_one_byte_over() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("over.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        let data = vec![b'x'; 1025];
        f.write_all(&data).unwrap();
        drop(f);

        assert!(is_big_file(path.to_str().unwrap(), 1024));
    }
}
