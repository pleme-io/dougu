//! Zen — distraction-free editing mode.
//!
//! Toggles a minimal editing environment by hiding UI elements like the
//! statusline, tab bar, line numbers, sign column, and fold column.

use std::sync::Mutex;

use nvim_oxi::api;
use nvim_oxi::api::opts::OptionOpts;

/// Saved option values for restoring after zen mode.
#[derive(Debug, Clone)]
struct SavedState {
    number: bool,
    relativenumber: bool,
    signcolumn: String,
    foldcolumn: String,
    laststatus: i64,
    showtabline: i64,
    cmdheight: i64,
    ruler: bool,
    showcmd: bool,
    showmode: bool,
}

/// Global zen state. `Some` when zen mode is active.
static ZEN_STATE: Mutex<Option<SavedState>> = Mutex::new(None);

/// Check if zen mode is currently active.
#[must_use]
pub fn is_active() -> bool {
    ZEN_STATE.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

fn get_global_opt<T: nvim_oxi::conversion::FromObject>(name: &str) -> Result<T, nvim_oxi::Error> {
    let opts = OptionOpts::builder().build();
    api::get_option_value::<T>(name, &opts).map_err(Into::into)
}

fn set_global_opt<T: nvim_oxi::conversion::ToObject>(
    name: &str,
    value: T,
) -> Result<(), nvim_oxi::Error> {
    let opts = OptionOpts::builder().build();
    api::set_option_value(name, value, &opts).map_err(Into::into)
}

/// Enter zen mode: save current options and set minimal UI.
fn enter_zen() -> Result<(), nvim_oxi::Error> {
    let state = SavedState {
        number: get_global_opt("number")?,
        relativenumber: get_global_opt("relativenumber")?,
        signcolumn: get_global_opt("signcolumn")?,
        foldcolumn: get_global_opt("foldcolumn")?,
        laststatus: get_global_opt("laststatus")?,
        showtabline: get_global_opt("showtabline")?,
        cmdheight: get_global_opt("cmdheight")?,
        ruler: get_global_opt("ruler")?,
        showcmd: get_global_opt("showcmd")?,
        showmode: get_global_opt("showmode")?,
    };

    // Disable UI elements
    set_global_opt("number", false)?;
    set_global_opt("relativenumber", false)?;
    set_global_opt("signcolumn", "no")?;
    set_global_opt("foldcolumn", "0")?;
    set_global_opt("laststatus", 0i64)?;
    set_global_opt("showtabline", 0i64)?;
    set_global_opt("cmdheight", 0i64)?;
    set_global_opt("ruler", false)?;
    set_global_opt("showcmd", false)?;
    set_global_opt("showmode", false)?;

    *ZEN_STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(state);

    api::command("echomsg '[dougu] zen mode on'")?;
    Ok(())
}

/// Leave zen mode: restore saved options.
fn leave_zen() -> Result<(), nvim_oxi::Error> {
    let state = ZEN_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .expect("leave_zen called while not in zen mode");

    set_global_opt("number", state.number)?;
    set_global_opt("relativenumber", state.relativenumber)?;
    set_global_opt("signcolumn", state.signcolumn.as_str())?;
    set_global_opt("foldcolumn", state.foldcolumn.as_str())?;
    set_global_opt("laststatus", state.laststatus)?;
    set_global_opt("showtabline", state.showtabline)?;
    set_global_opt("cmdheight", state.cmdheight)?;
    set_global_opt("ruler", state.ruler)?;
    set_global_opt("showcmd", state.showcmd)?;
    set_global_opt("showmode", state.showmode)?;

    api::command("echomsg '[dougu] zen mode off'")?;
    Ok(())
}

/// Toggle zen mode on or off.
pub fn toggle_zen() -> Result<(), tane::Error> {
    if is_active() {
        leave_zen().map_err(tane::Error::Oxi)?;
    } else {
        enter_zen().map_err(tane::Error::Oxi)?;
    }
    Ok(())
}
