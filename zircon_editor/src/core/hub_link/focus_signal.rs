use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime_interface::hub_protocol::{
    hub_editor_focus_signal_path, HubEditorFocusSignalV1, HubSessionToken,
};

use crate::core::recovery::SessionLockRecord;

use super::HubFocusSignalError;

/// Publishes one attention request for the editor session described by the live lock record.
///
/// The session lock remains the only liveness authority. This mailbox is deliberately separate
/// so Hub cannot mutate recovery data while forwarding a focus request.
pub fn publish_focus_signal(
    project_root: impl AsRef<Path>,
    target: &SessionLockRecord,
    session: HubSessionToken,
) -> Result<(), HubFocusSignalError> {
    let path = focus_signal_path(project_root.as_ref(), target.instance_id())?;
    let bytes = serde_json::to_vec(&HubEditorFocusSignalV1::new(session, target.instance_id()))
        .map_err(|source| HubFocusSignalError::Encode {
            path: path.clone(),
            source,
        })?;
    atomic_write(&path, &bytes).map_err(|source| HubFocusSignalError::Io {
        operation: "publish",
        path,
        source,
    })
}

/// Atomically claims and consumes a matching attention request, if one is present.
///
/// A malformed or mismatched request remains under a private claimed name. This prevents a
/// watcher from repeatedly surfacing the same invalid data while preserving it for diagnosis.
pub fn consume_focus_signal(
    project_root: impl AsRef<Path>,
    local_instance_id: &str,
) -> Result<Option<HubSessionToken>, HubFocusSignalError> {
    let mailbox_path = focus_signal_path(project_root.as_ref(), local_instance_id)?;
    let Some(claimed_path) = claim_focus_signal(&mailbox_path)? else {
        return Ok(None);
    };

    let bytes = fs::read(&claimed_path).map_err(|source| HubFocusSignalError::Io {
        operation: "read claimed",
        path: claimed_path.clone(),
        source,
    })?;
    let signal = serde_json::from_slice::<HubEditorFocusSignalV1>(&bytes).map_err(|source| {
        HubFocusSignalError::Decode {
            path: claimed_path.clone(),
            source,
        }
    })?;
    if signal.target_instance_id != local_instance_id {
        return Err(HubFocusSignalError::TargetMismatch {
            path: claimed_path,
            expected_instance_id: local_instance_id.to_string(),
            actual_instance_id: signal.target_instance_id,
        });
    }

    fs::remove_file(&claimed_path).map_err(|source| HubFocusSignalError::Io {
        operation: "complete consumption of",
        path: claimed_path,
        source,
    })?;
    Ok(Some(signal.session))
}

pub fn focus_signal_path(
    project_root: impl AsRef<Path>,
    instance_id: &str,
) -> Result<PathBuf, HubFocusSignalError> {
    hub_editor_focus_signal_path(project_root, instance_id).map_err(Into::into)
}

fn claim_path(mailbox_path: &Path) -> PathBuf {
    static NEXT_FOCUS_CLAIM: AtomicU64 = AtomicU64::new(1);

    let sequence = NEXT_FOCUS_CLAIM.fetch_add(1, Ordering::Relaxed);
    let file_name = mailbox_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("focus.json");
    mailbox_path.with_file_name(format!(
        ".{file_name}.zr-focus-claim-{}-{sequence}",
        std::process::id()
    ))
}

fn claim_focus_signal(mailbox_path: &Path) -> Result<Option<PathBuf>, HubFocusSignalError> {
    loop {
        let claimed_path = claim_path(mailbox_path);
        match fs::rename(mailbox_path, &claimed_path) {
            Ok(()) => return Ok(Some(claimed_path)),
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(HubFocusSignalError::Io {
                    operation: "claim",
                    path: mailbox_path.to_path_buf(),
                    source,
                });
            }
        }
    }
}
