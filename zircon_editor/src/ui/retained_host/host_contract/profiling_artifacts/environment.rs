use std::path::PathBuf;

pub(in crate::ui::retained_host::host_contract) fn profile_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE")
}

pub(in crate::ui::retained_host::host_contract) fn profile_screenshot_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE_SCREENSHOTS")
}

pub(in crate::ui::retained_host::host_contract) fn is_forced_softbuffer_screenshot_run() -> bool {
    env_truthy("ZIRCON_PROFILE_FORCE_SOFTBUFFER") && !profile_capture_enabled()
}

pub(in crate::ui::retained_host::host_contract) fn profile_export_dir() -> Option<PathBuf> {
    let output_root = std::env::var("ZIRCON_PROFILE_OUTPUT_ROOT").ok()?;
    let session_id = std::env::var("ZIRCON_PROFILE_SESSION").unwrap_or_else(|_| "local".into());
    Some(PathBuf::from(output_root).join(sanitize_session_id(&session_id)))
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

fn sanitize_session_id(session_id: &str) -> String {
    session_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
