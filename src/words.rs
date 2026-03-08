//! Words — highlight all occurrences of the word under the cursor.
//!
//! Installs a `CursorMoved` autocmd that highlights every occurrence of the
//! word under the cursor using extmarks in a dedicated namespace.

use nvim_oxi::api;
use nvim_oxi::api::opts::SetExtmarkOpts;
use tane::prelude::*;

/// The highlight group used for word highlighting.
const HL_GROUP: &str = "DouguWordHighlight";

/// Namespace name for word extmarks.
const NS_NAME: &str = "dougu_words";

/// Set up the words highlight group and autocmd.
pub fn setup() -> Result<(), tane::Error> {
    // Define the highlight group — links to `CurSearch` by default.
    Highlight::new(HL_GROUP).link("CurSearch").apply()?;

    // Create namespace
    let ns_id = api::create_namespace(NS_NAME);

    // Register CursorMoved autocmd
    Autocmd::on(&["CursorMoved", "CursorMovedI"])
        .pattern("*")
        .group("dougu_words")
        .desc("Dougu: highlight word under cursor")
        .register(move |_args| {
            highlight_word_under_cursor(ns_id).unwrap_or(());
            Ok(false)
        })?;

    Ok(())
}

/// Extract the word under the cursor using Neovim's `expand('<cword>')`.
fn get_word_under_cursor() -> Result<String, nvim_oxi::Error> {
    let word: String = api::call_function("expand", ("cword!",))?;
    Ok(word)
}

/// Highlight all occurrences of the word under the cursor in the current
/// buffer.
fn highlight_word_under_cursor(ns_id: u32) -> Result<(), nvim_oxi::Error> {
    let mut buf = api::get_current_buf();

    // Clear previous highlights
    buf.clear_namespace(ns_id, ..)?;

    let word = get_word_under_cursor()?;
    if word.is_empty() {
        return Ok(());
    }

    let line_count = buf.line_count()?;
    if line_count == 0 {
        return Ok(());
    }

    // Get all lines in the buffer and search for the word
    let lines: Vec<String> = buf
        .get_lines(0..line_count, false)?
        .map(|s| s.to_string_lossy().into())
        .collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let mut start = 0;
        while let Some(pos) = line[start..].find(&word) {
            let col = start + pos;
            let end_col = col + word.len();

            // Only highlight whole words: check boundaries
            let before_ok = col == 0
                || !line.as_bytes()[col - 1].is_ascii_alphanumeric()
                    && line.as_bytes()[col - 1] != b'_';
            let after_ok = end_col >= line.len()
                || !line.as_bytes()[end_col].is_ascii_alphanumeric()
                    && line.as_bytes()[end_col] != b'_';

            if before_ok && after_ok {
                let opts = SetExtmarkOpts::builder()
                    .end_col(end_col)
                    .hl_group(HL_GROUP)
                    .build();
                let _ = buf.set_extmark(ns_id, line_idx, col, &opts);
            }

            start = col + word.len().max(1);
        }
    }

    Ok(())
}
