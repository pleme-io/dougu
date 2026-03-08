//! Dashboard — simple startup screen with ASCII art, recent files, and
//! keybind hints.
//!
//! Displays a centered dashboard in a scratch buffer when Neovim starts with
//! no file arguments.

use nvim_oxi::api;
use nvim_oxi::api::opts::OptionOpts;

/// ASCII art header for the dashboard.
const HEADER: &[&str] = &[
    r"      ___           ___           ___           ___           ___     ",
    r"     /\  \         /\  \         /\__\         /\  \         /\__\    ",
    r"    /::\  \       /::\  \       /:/  /        /::\  \       /:/  /    ",
    r"   /:/\:\  \     /:/\:\  \     /:/  /        /:/\:\  \     /:/  /     ",
    r"  /:/  \:\__\   /:/  \:\  \   /:/  /  ___   /:/  \:\  \   /:/  /  ___ ",
    r" /:/__/ \:|__| /:/__/ \:\__\ /:/__/  /\__\ /:/__/_\:\__\ /:/__/  /\__\",
    r" \:\  \ /:/  / \:\  \ /:/  / \:\  \ /:/  / \:\  /\ \/__/ \:\  \ /:/  /",
    r"  \:\  /:/  /   \:\  /:/  /   \:\  /:/  /   \:\ \:\__\    \:\  /:/  / ",
    r"   \:\/:/  /     \:\/:/  /     \:\/:/  /     \:\/:/  /     \:\/:/  /  ",
    r"    \::/__/       \::/  /       \::/  /       \::/  /       \::/  /   ",
    r"     ~~            \/__/         \/__/         \/__/         \/__/    ",
    "",
    "                         道具 — dougu",
    "",
];

/// Keybind hints shown on the dashboard.
const HINTS: &[(&str, &str)] = &[
    ("e", "New file"),
    ("f", "Find file"),
    ("r", "Recent files"),
    ("s", "Scratch buffer"),
    ("q", "Quit"),
];

/// Show the dashboard in a new scratch buffer.
pub fn show_dashboard() -> Result<(), tane::Error> {
    // Create a new scratch buffer
    let mut buf = api::create_buf(true, true)?;

    // Switch to it
    api::set_current_buf(&buf)?;

    // Set buffer options
    let buf_opts = OptionOpts::builder().buffer(buf.clone()).build();
    api::set_option_value("bufhidden", "wipe", &buf_opts)?;
    api::set_option_value("buftype", "nofile", &buf_opts)?;
    api::set_option_value("swapfile", false, &buf_opts)?;
    api::set_option_value("modifiable", true, &buf_opts)?;

    // Build content lines
    let mut lines: Vec<String> = Vec::new();

    // Add some top padding
    for _ in 0..3 {
        lines.push(String::new());
    }

    // Add header
    for line in HEADER {
        lines.push((*line).to_string());
    }

    // Add hints
    lines.push(String::new());
    for (key, desc) in HINTS {
        lines.push(format!("    [{key}]  {desc}"));
    }
    lines.push(String::new());

    // Add recent files heading
    lines.push("  Recent Files".to_string());
    lines.push("  ────────────".to_string());

    // Try to get oldfiles (recent files)
    let recent: Vec<String> = api::get_vvar::<Vec<String>>("oldfiles").unwrap_or_default();
    let display_count = recent.len().min(8);
    for file in recent.iter().take(display_count) {
        lines.push(format!("    {file}"));
    }
    if recent.is_empty() {
        lines.push("    (no recent files)".to_string());
    }

    // Set lines into the buffer
    buf.set_lines(0.., false, lines)?;

    // Make buffer non-modifiable
    api::set_option_value("modifiable", false, &buf_opts)?;

    // Set window options for a clean look
    let win_opts = OptionOpts::builder()
        .scope(nvim_oxi::api::opts::OptionScope::Local)
        .build();
    api::set_option_value("number", false, &win_opts)?;
    api::set_option_value("relativenumber", false, &win_opts)?;
    api::set_option_value("cursorline", false, &win_opts)?;
    api::set_option_value("signcolumn", "no", &win_opts)?;
    api::set_option_value("foldcolumn", "0", &win_opts)?;

    Ok(())
}
