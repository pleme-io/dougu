//! Toggle — toggle boolean Neovim options.
//!
//! Provides `:DouguToggle <option>` to flip boolean options like `wrap`,
//! `number`, `relativenumber`, `spell`, `list`, `cursorline`, `cursorcolumn`,
//! etc.

use nvim_oxi::api;
use nvim_oxi::api::opts::OptionOpts;

/// Known boolean options that can be toggled.
pub const TOGGLEABLE_OPTIONS: &[&str] = &[
    "wrap",
    "number",
    "relativenumber",
    "spell",
    "list",
    "cursorline",
    "cursorcolumn",
    "ignorecase",
    "smartcase",
    "expandtab",
    "hlsearch",
    "incsearch",
    "ruler",
    "showmode",
    "hidden",
    "backup",
    "writebackup",
    "swapfile",
    "undofile",
    "termguicolors",
    "splitbelow",
    "splitright",
    "autowrite",
    "autoread",
    "confirm",
    "showcmd",
    "showmatch",
    "linebreak",
    "breakindent",
    "foldenable",
    "lazyredraw",
    "signcolumn_toggle", // special-cased below
];

/// Check whether the given option name is a known toggleable boolean option.
#[must_use]
pub fn is_toggleable(option: &str) -> bool {
    TOGGLEABLE_OPTIONS.contains(&option)
}

/// Get the current boolean value of an option.
pub fn get_option_bool(name: &str) -> Result<bool, nvim_oxi::Error> {
    let opts = OptionOpts::builder().build();
    api::get_option_value::<bool>(name, &opts).map_err(Into::into)
}

/// Set a boolean option value.
pub fn set_option_bool(name: &str, value: bool) -> Result<(), nvim_oxi::Error> {
    let opts = OptionOpts::builder().build();
    api::set_option_value(name, value, &opts).map_err(Into::into)
}

/// Toggle the given option. Returns the new value as a display string.
pub fn toggle_option(option: &str) -> Result<(), tane::Error> {
    let current = get_option_bool(option).map_err(tane::Error::Oxi)?;
    let new_value = !current;
    set_option_bool(option, new_value).map_err(tane::Error::Oxi)?;

    let state = if new_value { "on" } else { "off" };
    let msg = format!("echomsg '[dougu] {option}: {state}'");
    api::command(&msg)?;

    Ok(())
}

/// Parse a toggle command argument, validating the option name.
///
/// Returns the option name if valid, or an error description.
#[must_use]
pub fn parse_toggle_arg(arg: &str) -> Result<&str, &'static str> {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return Err("no option name provided");
    }
    // We allow any option name — Neovim will error if it doesn't exist.
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_options_are_toggleable() {
        assert!(is_toggleable("wrap"));
        assert!(is_toggleable("number"));
        assert!(is_toggleable("spell"));
        assert!(is_toggleable("relativenumber"));
        assert!(is_toggleable("cursorline"));
    }

    #[test]
    fn unknown_options_are_not_toggleable() {
        assert!(!is_toggleable("shiftwidth"));
        assert!(!is_toggleable("tabstop"));
        assert!(!is_toggleable("nonexistent"));
    }

    #[test]
    fn parse_toggle_arg_with_valid_input() {
        assert_eq!(parse_toggle_arg("wrap"), Ok("wrap"));
        assert_eq!(parse_toggle_arg("  number  "), Ok("number"));
    }

    #[test]
    fn parse_toggle_arg_with_empty_input() {
        assert_eq!(parse_toggle_arg(""), Err("no option name provided"));
        assert_eq!(parse_toggle_arg("   "), Err("no option name provided"));
    }

    #[test]
    fn parse_toggle_arg_allows_unknown_options() {
        // We allow any option name; Neovim validates at runtime.
        assert_eq!(parse_toggle_arg("customopt"), Ok("customopt"));
    }

    #[test]
    fn toggleable_options_list_not_empty() {
        assert!(!TOGGLEABLE_OPTIONS.is_empty());
    }

    #[test]
    fn toggleable_options_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for opt in TOGGLEABLE_OPTIONS {
            assert!(seen.insert(*opt), "Duplicate option: {opt}");
        }
    }
}
