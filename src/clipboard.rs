use anyhow::{Context, Result};
use arboard::Clipboard;
use std::thread;
use std::time::Duration;

const CLEAR_AFTER: Duration = Duration::from_secs(45);

/// Copies `text` to the clipboard, then blocks until `CLEAR_AFTER` elapses
/// and clears it, so a password doesn't linger there indefinitely. Skips
/// the clear if the clipboard no longer holds `text` (the user copied
/// something else in the meantime).
pub(crate) fn copy_with_timeout(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("opening clipboard")?;
    clipboard
        .set_text(text.to_string())
        .context("copying to clipboard")?;
    println!(
        "copied to clipboard, clearing in {}s (ctrl-c to exit without waiting)",
        CLEAR_AFTER.as_secs()
    );

    thread::sleep(CLEAR_AFTER);
    if clipboard.get_text().ok().as_deref() == Some(text) {
        clipboard.clear().context("clearing clipboard")?;
    }
    Ok(())
}
