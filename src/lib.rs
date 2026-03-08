//! Dougu (道具) — utility toolkit for Neovim: dashboard, terminal, zen mode, scroll animations
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.

use nvim_oxi as oxi;

#[oxi::plugin]
fn dougu() -> oxi::Result<()> {
    Ok(())
}
