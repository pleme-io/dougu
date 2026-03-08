//! Quickfile — skip the startup screen and go directly to a file.
//!
//! If Neovim was launched with a file argument, this module skips the intro
//! screen and loads the file immediately by triggering `BufReadPost` and
//! filetype detection early.

use nvim_oxi::api;
use tane::prelude::*;

/// Set up quickfile behavior.
///
/// On `BufReadPost`, if the current buffer has a file name (meaning Neovim was
/// invoked with a file argument), trigger filetype detection immediately.
pub fn setup() -> Result<(), tane::Error> {
    Autocmd::on(&["BufReadPost"])
        .pattern("*")
        .group("dougu_quickfile")
        .desc("Dougu: trigger filetype detection on file open")
        .once()
        .register(|_args| {
            let buf = api::get_current_buf();
            let name = buf.get_name().unwrap_or_default();
            if !name.as_os_str().is_empty() {
                let _ = api::command("filetype detect");
            }
            Ok(false)
        })?;
    Ok(())
}
