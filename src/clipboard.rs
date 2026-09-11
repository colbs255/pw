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
    use std::sync::mpsc;
    use std::thread;

    // `SetExtLinux::wait_until` only really blocks on X11: arboard's Wayland
    // backend treats any deadline other than "forever" as "don't wait at all",
    // so it hands the clipboard off to a thread and returns immediately. If
    // this function then returned too, the process would exit and take that
    // thread down with it before anyone could paste. Instead, run the
    // (properly-blocking, on both backends) `wait()` on its own thread and
    // bound it ourselves, so exiting this process is always what ends up
    // clearing the clipboard, whether that's because AVAILABLE_FOR elapsed or
    // because the clipboard was overwritten first.
    let text = text.to_string();
    let (overwritten_tx, overwritten_rx) = mpsc::channel();
    let server = thread::spawn(move || -> Result<()> {
        let mut clipboard = Clipboard::new().context("opening clipboard")?;
        let result = clipboard
            .set()
            .wait()
            .text(text)
            .context("copying to clipboard");
        let _ = overwritten_tx.send(());
        result
    });

    println!(
        "copied to clipboard, available for {}s or until overwritten (ctrl-c to exit early)",
        AVAILABLE_FOR.as_secs()
    );

    if overwritten_rx.recv_timeout(AVAILABLE_FOR).is_ok() {
        return server.join().expect("clipboard server thread panicked");
    }
    Ok(())
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
