use std::path::Path;

pub(super) fn show_failure(args: &[String], diagnostic: &str, log_path: Option<&Path>) {
    #[cfg(all(windows, feature = "shipping-editor"))]
    {
        let preference = std::env::var("ZIRCON_EDITOR_STARTUP_DIALOG").ok();
        let first_frame_exit = std::env::var("ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME")
            .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"));
        if !should_show_dialog(args, preference.as_deref(), first_frame_exit) {
            return;
        }
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK, MB_SETFOREGROUND};
        let text = wide_message(&failure_message(diagnostic, log_path));
        let title = wide_message("Zircon Editor startup failed");
        // The buffers are NUL-terminated UTF-16 and live until the synchronous native call returns.
        unsafe {
            MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR | MB_SETFOREGROUND);
        }
    }
    #[cfg(not(all(windows, feature = "shipping-editor")))]
    let _ = (args, diagnostic, log_path);
}

#[cfg(any(test, all(windows, feature = "shipping-editor")))]
fn should_show_dialog(args: &[String], preference: Option<&str>, first_frame_exit: bool) -> bool {
    let disabled = preference.is_some_and(|value| {
        ["0", "false", "off", "no"].iter().any(|disabled| value.eq_ignore_ascii_case(disabled))
    });
    !disabled && !first_frame_exit && !args.iter().any(|arg| {
        matches!(arg.as_str(), "-h" | "--help" | "--run") || arg.starts_with("--run=")
    })
}

#[cfg(any(test, all(windows, feature = "shipping-editor")))]
fn failure_message(diagnostic: &str, log_path: Option<&Path>) -> String {
    let log = log_path.map_or_else(|| "Log file unavailable.".to_owned(), |path| format!("Log: {}", path.display()));
    format!("{diagnostic}\n\n{log}")
}

#[cfg(all(windows, feature = "shipping-editor"))]
fn wide_message(text: &str) -> Vec<u16> {
    text.replace('\0', "�").encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
#[path = "../tests/editor_startup_diagnostic.rs"]
mod tests;
