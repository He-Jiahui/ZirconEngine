use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use zircon_runtime_interface::hub_protocol::{HubEditorMailboxV1, HubSessionToken};

use crate::error::HubError;

pub(super) fn read_editor_handshake(
    mailbox_path: &Path,
    expected_session: HubSessionToken,
) -> Result<Option<HubEditorMailboxV1>, HubError> {
    match fs::read(mailbox_path) {
        Ok(bytes) => {
            let mailbox = serde_json::from_slice::<HubEditorMailboxV1>(&bytes)?;
            mailbox
                .validate_launch_session(expected_session)
                .map_err(|error| HubError::message(error.to_string()))?;
            Ok(Some(mailbox))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
