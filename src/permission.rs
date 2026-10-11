//! Accessibility permission, required for the event tap and for reading the Dock's position.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

use crate::ffi;

/// Opens the Accessibility page of System Settings.
pub fn open_settings() {
    let _ = std::process::Command::new("/usr/bin/open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status();
}

/// Checks the permission without showing a dialog.
pub fn is_trusted() -> bool {
    unsafe { ffi::AXIsProcessTrusted() != 0 }
}

/// Checks the permission and lets macOS show its "grant access" dialog if it is missing.
/// The dialog does not block: the answer is the state at the time of the call.
pub fn request() -> bool {
    unsafe {
        let key = CFString::wrap_under_get_rule(ffi::kAXTrustedCheckOptionPrompt);
        let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
        ffi::AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) != 0
    }
}

/// Name of the terminal app this command runs in, from `TERM_PROGRAM`. macOS gives the Accessibility
/// permission of a command started from a terminal to that app, not to dockjail.
pub fn terminal_name() -> String {
    let name = match std::env::var("TERM_PROGRAM").as_deref() {
        Ok("Apple_Terminal") => "Terminal",
        Ok("iTerm.app") => "iTerm",
        Ok("WarpTerminal") => "Warp",
        Ok("ghostty") => "Ghostty",
        Ok("WezTerm") => "WezTerm",
        Ok("vscode") => "your editor (VS Code or a fork of it)",
        _ => "your terminal app",
    };
    name.to_string()
}

/// Steps for granting the permission to the terminal app.
pub fn terminal_hint() -> String {
    let terminal = terminal_name();
    format!(
        "This command runs in {terminal}, so {terminal} needs the permission, not dockjail:\n  \
         1. Enable {terminal} in System Settings → Privacy & Security → Accessibility.\n     \
         If it is not listed, add it with \"+\" from /Applications.\n  \
         2. Quit {terminal} completely (Cmd+Q) and open it again; macOS applies the permission after a restart.\n  \
         3. Run `dockjail check` again."
    )
}

/// Steps for granting the permission to the login agent, which runs the dockjail binary itself.
pub fn agent_hint() -> String {
    // The real file, not the path it was started by: Homebrew's bin/dockjail is only a symlink.
    let binary = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "the dockjail binary".to_string());
    format!(
        "The login agent runs without a terminal, so it needs its own entry, separate from {}:\n  \
         1. Enable `dockjail` in System Settings → Privacy & Security → Accessibility.\n     \
         If it is not listed, click \"+\", press Cmd+Shift+G and enter {binary}\n  \
         2. The agent retries every 30 seconds. `dockjail log` shows whether it started.",
        terminal_name()
    )
}
