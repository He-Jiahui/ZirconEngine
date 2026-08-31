use std::path::Path;
use std::time::{Duration, Instant};

use zircon_runtime_interface::hub_protocol::{HubEditorMailboxV1, HubSessionToken};

use crate::error::HubError;

use super::mailbox_path::editor_handshake_mailbox_path;
use super::read::read_editor_handshake;

const HUB_HANDSHAKE_POLL_INTERVAL: Duration = Duration::from_millis(250);
const HUB_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Waits for a terminal Editor mailbox response. Call this only from a Hub background task.
pub(crate) fn wait_for_editor_handshake(
    project_root: impl AsRef<Path>,
    session: HubSessionToken,
) -> Result<HubEditorMailboxV1, HubError> {
    let mailbox_path = editor_handshake_mailbox_path(project_root, session);
    wait_for_editor_handshake_until(
        Instant::now() + HUB_HANDSHAKE_TIMEOUT,
        HUB_HANDSHAKE_POLL_INTERVAL,
        || read_editor_handshake(&mailbox_path, session),
    )
}

pub(super) fn wait_for_editor_handshake_until<F>(
    deadline: Instant,
    poll_interval: Duration,
    mut read: F,
) -> Result<HubEditorMailboxV1, HubError>
where
    F: FnMut() -> Result<Option<HubEditorMailboxV1>, HubError>,
{
    while Instant::now() < deadline {
        if let Some(mailbox) = read()? {
            return Ok(mailbox);
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        std::thread::sleep(poll_interval.min(remaining));
    }

    Err(HubError::message("editor Hub handshake timed out"))
}
