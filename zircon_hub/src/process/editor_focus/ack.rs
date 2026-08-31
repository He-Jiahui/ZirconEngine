use std::fs;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use zircon_runtime_interface::hub_protocol::{
    hub_editor_focus_ack_path, HubEditorFocusAckDispositionV1, HubEditorFocusAckV1,
    HubEditorFocusSignalV1,
};

use crate::error::HubError;

const FOCUS_ACK_POLL_INTERVAL: Duration = Duration::from_millis(25);
const FOCUS_ACK_MAX_BYTES: u64 = 4 * 1024;

/// Waits for the addressed editor owner to report the terminal outcome of one focus request.
///
/// This runs inside Hub's background action worker. It never blocks the Tauri event loop and
/// accepts only a receipt bound to the exact request identity written by Hub.
pub(crate) fn wait_for_project_editor_focus_ack(
    project_root: impl AsRef<Path>,
    request: &HubEditorFocusSignalV1,
) -> Result<(), HubError> {
    let acknowledgement_path = hub_editor_focus_ack_path(
        project_root,
        &request.target_instance_id,
        request.sequence,
        request.request_id,
    )
    .map_err(|source| HubError::message(source.to_string()))?;

    loop {
        let now_unix_millis = unix_millis_now()?;
        if request.is_expired_at(now_unix_millis) {
            return Err(HubError::message(
                "editor focus acknowledgement did not arrive before the request deadline",
            ));
        }

        match fs::read(&acknowledgement_path) {
            Ok(bytes) => {
                if bytes.len() as u64 > FOCUS_ACK_MAX_BYTES {
                    return Err(HubError::message(
                        "editor focus acknowledgement exceeds the bounded mailbox byte limit",
                    ));
                }
                let acknowledgement = serde_json::from_slice::<HubEditorFocusAckV1>(&bytes)?;
                let result = validate_focus_acknowledgement(request, &acknowledgement);
                if result.is_ok() || acknowledgement.matches_request(request) {
                    fs::remove_file(&acknowledgement_path)?;
                }
                return result;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                thread::sleep(FOCUS_ACK_POLL_INTERVAL);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn validate_focus_acknowledgement(
    request: &HubEditorFocusSignalV1,
    acknowledgement: &HubEditorFocusAckV1,
) -> Result<(), HubError> {
    if !acknowledgement.matches_request(request) {
        return Err(HubError::message(
            "editor focus acknowledgement is not bound to the Hub focus request",
        ));
    }
    match acknowledgement.disposition {
        HubEditorFocusAckDispositionV1::Focused => Ok(()),
        HubEditorFocusAckDispositionV1::RejectedExpired => Err(HubError::message(
            "editor rejected the Hub focus request because its deadline expired",
        )),
        HubEditorFocusAckDispositionV1::RejectedTargetMismatch => Err(HubError::message(
            "editor rejected the Hub focus request because its session target changed",
        )),
        HubEditorFocusAckDispositionV1::RejectedInboxFull => Err(HubError::message(
            "editor rejected the Hub focus request because its bounded focus inbox is full",
        )),
        HubEditorFocusAckDispositionV1::RejectedStale => Err(HubError::message(
            "editor rejected the Hub focus request because its project session was retired",
        )),
    }
}

fn unix_millis_now() -> Result<u64, HubError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .map_err(|error| HubError::message(format!("read Hub focus clock: {error}")))
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::hub_protocol::{HubEditorFocusAckV1, HubSessionToken};

    use super::validate_focus_acknowledgement;
    use zircon_runtime_interface::hub_protocol::{
        HubEditorFocusAckDispositionV1, HubEditorFocusSignalV1,
    };

    #[test]
    fn acknowledgement_requires_the_exact_request_identity_and_a_focused_disposition() {
        let request = HubEditorFocusSignalV1::new(HubSessionToken::new(), "913-42", 1, 7, u64::MAX)
            .expect("valid request");
        assert!(
            validate_focus_acknowledgement(&request, &HubEditorFocusAckV1::focused(&request))
                .is_ok()
        );

        let rejected = HubEditorFocusAckV1::from_request(
            &request,
            HubEditorFocusAckDispositionV1::RejectedExpired,
        );
        assert!(validate_focus_acknowledgement(&request, &rejected).is_err());
    }
}
