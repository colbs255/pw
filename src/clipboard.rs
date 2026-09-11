use anyhow::{Context, Result};
use arboard::Clipboard;
use std::time::Duration;

const AVAILABLE_FOR: Duration = Duration::from_secs(45);

/// Copies `text` to the clipboard.
///
/// On Linux/BSD the clipboard is "hosted" by the process that set it, and
/// its contents disappear the moment that process exits, so this blocks
/// and keeps serving paste requests for `AVAILABLE_FOR`, or until the user
/// copies something else, whichever comes first. macOS and Windows own the
/// clipboard at the OS level and don't have this problem, so there this
/// copies and returns immediately.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub(crate) fn copy(text: &str) -> Result<()> {
    use arboard::SetExtLinux;
    use std::time::Instant;

    let mut clipboard = Clipboard::new().context("opening clipboard")?;
    println!(
        "copied to clipboard, available for {}s or until overwritten (ctrl-c to exit early)",
        AVAILABLE_FOR.as_secs()
    );
    clipboard
        .set()
        .wait_until(Instant::now() + AVAILABLE_FOR)
        .text(text.to_string())
        .context("copying to clipboard")
}

#[cfg(not(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub(crate) fn copy(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("opening clipboard")?;
    clipboard
        .set_text(text.to_string())
        .context("copying to clipboard")?;
    println!("copied to clipboard");
    Ok(())
}
