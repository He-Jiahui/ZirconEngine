use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use zircon_runtime_interface::hub_protocol::HubEditorMailboxV1;

use crate::error::HubError;

pub(super) fn read_editor_handshake(
    mailbox_path: &Path,
) -> Result<Option<HubEditorMailboxV1>, HubError> {
    match fs::read(mailbox_path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
