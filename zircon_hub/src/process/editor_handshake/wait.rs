use std::path::Path;
use std::time::Duration;

use zircon_runtime_interface::hub_protocol::{HubEditorMailboxV1, HubSessionToken};

use crate::error::HubError;
use crate::process::SupervisedChild;
use crate::state::{TaskCancellationToken, TaskExecutionOutcome};

use super::mailbox_path::editor_handshake_mailbox_path;
use super::read::read_editor_handshake;

const HUB_HANDSHAKE_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Supervises the Editor until it publishes a terminal mailbox or the child actually exits.
/// Call this only from a Hub background task.
pub(crate) fn wait_for_editor_handshake(
    project_root: impl AsRef<Path>,
    session: HubSessionToken,
    child: &mut SupervisedChild,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<HubEditorMailboxV1>, HubError> {
    let mailbox_path = editor_handshake_mailbox_path(project_root, session);
    wait_for_editor_handshake_until_terminal(
        HUB_HANDSHAKE_POLL_INTERVAL,
        || read_editor_handshake(&mailbox_path, session),
        || poll_supervised_child(child, cancellation),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SupervisedChildState {
    Running,
    Exited(String),
    CancellationRequested,
}

pub(super) fn wait_for_editor_handshake_until_terminal<F, S>(
    poll_interval: Duration,
    mut read: F,
    mut child_terminal: S,
) -> Result<TaskExecutionOutcome<HubEditorMailboxV1>, HubError>
where
    F: FnMut() -> Result<Option<HubEditorMailboxV1>, HubError>,
    S: FnMut() -> Result<SupervisedChildState, HubError>,
{
    loop {
        if let Some(mailbox) = read()? {
            return Ok(TaskExecutionOutcome::Completed(mailbox));
        }
        match child_terminal()? {
            SupervisedChildState::Running => {}
            SupervisedChildState::Exited(status) => {
                // The Editor may atomically publish its terminal mailbox between the first read
                // and `try_wait`. Re-read after observing exit so a completed handshake wins.
                if let Some(mailbox) = read()? {
                    return Ok(TaskExecutionOutcome::Completed(mailbox));
                }
                return Err(HubError::message(format!(
                    "editor process exited before publishing its Hub terminal handshake: {status}"
                )));
            }
            SupervisedChildState::CancellationRequested => {
                // A terminal mailbox committed before termination is authoritative even when the
                // cancellation request and process supervision observation raced.
                if let Some(mailbox) = read()? {
                    return Ok(TaskExecutionOutcome::Completed(mailbox));
                }
                return Ok(TaskExecutionOutcome::Cancelled);
            }
        }
        std::thread::sleep(poll_interval);
    }
}

fn poll_supervised_child(
    child: &mut SupervisedChild,
    cancellation: &TaskCancellationToken,
) -> Result<SupervisedChildState, HubError> {
    if let Some(status) = child.try_wait()? {
        return Ok(SupervisedChildState::Exited(status.to_string()));
    }
    if cancellation.is_cancellation_requested() {
        return Ok(SupervisedChildState::CancellationRequested);
    }
    Ok(SupervisedChildState::Running)
}
