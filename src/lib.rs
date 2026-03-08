//! Dougu (道具) — utility toolkit for Neovim: quality-of-life features.
//!
//! A Rust replacement for snacks.nvim, providing:
//! - **Bigfile** — disable heavy features for large files
//! - **Quickfile** — skip startup screen if a file arg is present
//! - **Scratch** — persistent scratch buffers
//! - **Toggle** — toggle boolean Neovim options
//! - **Zen** — distraction-free editing mode
//! - **Words** — highlight all occurrences of word under cursor
//! - **Dashboard** — simple startup screen with recent files
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.

pub mod bigfile;
pub mod dashboard;
pub mod quickfile;
pub mod scratch;
pub mod toggle;
pub mod words;
pub mod zen;

use nvim_oxi as oxi;
use tane::prelude::*;

/// Convert a `tane::Error` into an `oxi::Error` by going through the API
/// error variant.
fn tane_to_oxi(err: tane::Error) -> oxi::Error {
    oxi::Error::Api(oxi::api::Error::Other(err.to_string()))
}

#[oxi::plugin]
fn dougu() -> oxi::Result<()> {
    // Register bigfile autocmd
    bigfile::setup().map_err(tane_to_oxi)?;

    // Register quickfile logic
    quickfile::setup().map_err(tane_to_oxi)?;

    // Register words autocmd
    words::setup().map_err(tane_to_oxi)?;

    // Register user commands
    UserCommand::new("DouguToggle")
        .one_arg()
        .desc("Toggle a Neovim option")
        .register(|args| {
            let arg = args.args.unwrap_or_default();
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(tane::Error::Custom(
                    "DouguToggle requires an option name".to_string(),
                ));
            }
            toggle::toggle_option(arg)?;
            Ok(())
        })
        .map_err(tane_to_oxi)?;

    UserCommand::new("DouguScratch")
        .optional_arg()
        .desc("Open or create a scratch buffer")
        .register(|args| {
            let name = args.args.unwrap_or_default();
            let name = name.trim().to_string();
            let name = if name.is_empty() { None } else { Some(name) };
            scratch::open_scratch(name.as_deref()).map_err(tane::Error::Oxi)?;
            Ok(())
        })
        .map_err(tane_to_oxi)?;

    UserCommand::new("DouguZen")
        .desc("Toggle zen (distraction-free) mode")
        .register(|_args| {
            zen::toggle_zen()?;
            Ok(())
        })
        .map_err(tane_to_oxi)?;

    UserCommand::new("DouguDashboard")
        .desc("Show the Dougu dashboard")
        .register(|_args| {
            dashboard::show_dashboard()?;
            Ok(())
        })
        .map_err(tane_to_oxi)?;

    Ok(())
}
