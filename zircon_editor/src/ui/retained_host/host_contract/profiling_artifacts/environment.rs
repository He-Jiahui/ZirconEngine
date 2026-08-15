use std::env::VarError;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("ZIRCON_PROFILE_OUTPUT_ROOT must be an absolute destination outside the C: system drive")]
pub(in crate::ui::retained_host::host_contract) struct ProfileOutputRootError;

pub(in crate::ui::retained_host::host_contract) fn profile_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE")
}

pub(in crate::ui::retained_host::host_contract) fn profile_screenshot_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE_SCREENSHOTS")
}

pub(in crate::ui::retained_host::host_contract) fn is_forced_softbuffer_screenshot_run() -> bool {
    env_truthy("ZIRCON_PROFILE_FORCE_SOFTBUFFER") && !profile_capture_enabled()
}

pub(in crate::ui::retained_host::host_contract) fn profile_export_dir(
) -> Result<Option<PathBuf>, ProfileOutputRootError> {
    let output_root = match std::env::var("ZIRCON_PROFILE_OUTPUT_ROOT") {
        Ok(value) => value,
        Err(VarError::NotPresent) => return Ok(None),
        Err(VarError::NotUnicode(_)) => return Err(ProfileOutputRootError),
    };
    let session_id = std::env::var("ZIRCON_PROFILE_SESSION").unwrap_or_else(|_| "local".into());
    profile_output_root(&output_root).map(|root| Some(root.join(sanitize_session_id(&session_id))))
}

fn profile_output_root(output_root: &str) -> Result<PathBuf, ProfileOutputRootError> {
    let root = PathBuf::from(output_root);
    let bytes = output_root.as_bytes();
    if bytes.starts_with(b"\\\\?\\") || bytes.starts_with(b"\\\\.\\") {
        return Err(ProfileOutputRootError);
    }
    if bytes.starts_with(b"\\\\") || bytes.starts_with(b"//") {
        return Ok(root);
    }
    let Some(drive) = bytes.get(..2) else {
        return Err(ProfileOutputRootError);
    };
    let is_absolute_drive_root = matches!(output_root.as_bytes().get(2), Some(b'\\' | b'/'));
    // Profile artifacts are operator outputs and must not consume the system drive.
    (is_absolute_drive_root && drive[1] == b':' && !drive[0].eq_ignore_ascii_case(&b'C'))
        .then_some(root)
        .ok_or(ProfileOutputRootError)
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

#[cfg(test)]
#[path = "environment/tests.rs"]
mod tests;
