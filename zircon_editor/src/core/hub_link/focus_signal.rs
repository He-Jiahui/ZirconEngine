use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime_interface::hub_protocol::{
    HubEditorFocusAckDispositionV1, HubEditorFocusAckV1, HubEditorFocusSignalV1, HubSessionToken,
    hub_editor_focus_ack_path, hub_editor_focus_request_directory, hub_editor_focus_signal_path,
};
use zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionLifecycleV1;

use crate::core::recovery::ProjectSessionAdmissionRecordV1;

use super::HubFocusSignalError;

const FOCUS_REQUEST_TTL_MILLIS: u64 = 10_000;
const FOCUS_REQUEST_MAX_BYTES: u64 = 4 * 1024;

static NEXT_FOCUS_REQUEST: AtomicU64 = AtomicU64::new(1);

/// Publishes one attention request for the editor session described by the live lock record.
///
/// The session lock remains the only liveness authority. This mailbox is deliberately separate
/// so Hub cannot mutate recovery data while forwarding a focus request.
pub fn publish_focus_signal(
    project_root: impl AsRef<Path>,
    target: &ProjectSessionAdmissionRecordV1,
    request_id: HubSessionToken,
) -> Result<(), HubFocusSignalError> {
    if target.lifecycle() != ProjectSessionAdmissionLifecycleV1::Ready
        || target.session_generation().is_none()
    {
        return Err(HubFocusSignalError::TargetNotReady {
            instance_id: target.instance_id().to_string(),
            lifecycle: target.lifecycle().as_str(),
        });
    }
    let now_unix_millis = unix_millis_now()?;
    let deadline_unix_millis = now_unix_millis
        .checked_add(FOCUS_REQUEST_TTL_MILLIS)
        .ok_or(HubFocusSignalError::DeadlineOverflow)?;
    let signal = HubEditorFocusSignalV1::new(
        request_id,
        target.instance_id(),
        target
            .session_generation()
            .expect("Ready focus target always has a committed generation")
            .get(),
        NEXT_FOCUS_REQUEST.fetch_add(1, Ordering::Relaxed),
        deadline_unix_millis,
    )?;
    let path = focus_signal_path(
        project_root.as_ref(),
        target.instance_id(),
        signal.sequence,
        signal.request_id,
    )?;
    let bytes = serde_json::to_vec(&signal).map_err(|source| HubFocusSignalError::Encode {
        path: path.clone(),
        source,
    })?;
    atomic_write(&path, &bytes).map_err(|source| HubFocusSignalError::Io {
        operation: "publish",
        path,
        source,
    })
}

/// Atomically claims and consumes matching attention requests in sequence order.
///
/// A malformed or instance-mismatched request remains under a private claimed name. This
/// prevents a watcher from repeatedly surfacing untrusted data while preserving it for
/// diagnosis. An otherwise valid request for a stale generation receives a typed rejection.
pub fn consume_focus_signals(
    project_root: impl AsRef<Path>,
    local_instance_id: &str,
    local_session_generation: u64,
) -> Result<Vec<HubEditorFocusSignalV1>, HubFocusSignalError> {
    let request_directory =
        hub_editor_focus_request_directory(project_root.as_ref(), local_instance_id)?;
    let mut request_paths = request_paths_in_sequence(&request_directory)?;
    let now_unix_millis = unix_millis_now()?;
    let mut consumed = Vec::with_capacity(request_paths.len());
    for request_path in request_paths.drain(..) {
        let Some(claimed_path) = claim_focus_signal(&request_path)? else {
            continue;
        };
        let bytes = fs::read(&claimed_path).map_err(|source| HubFocusSignalError::Io {
            operation: "read claimed",
            path: claimed_path.clone(),
            source,
        })?;
        if bytes.len() as u64 > FOCUS_REQUEST_MAX_BYTES {
            return Err(HubFocusSignalError::RequestTooLarge { path: claimed_path });
        }
        let signal =
            serde_json::from_slice::<HubEditorFocusSignalV1>(&bytes).map_err(|source| {
                HubFocusSignalError::Decode {
                    path: claimed_path.clone(),
                    source,
                }
            })?;
        signal.validate()?;
        if signal.target_instance_id != local_instance_id {
            return Err(HubFocusSignalError::TargetMismatch {
                path: claimed_path,
                expected_instance_id: local_instance_id.to_string(),
                actual_instance_id: signal.target_instance_id,
            });
        }
        if signal.target_session_generation != local_session_generation {
            publish_focus_ack(
                project_root.as_ref(),
                &signal,
                HubEditorFocusAckDispositionV1::RejectedStale,
            )?;
            fs::remove_file(&claimed_path).map_err(|source| HubFocusSignalError::Io {
                operation: "complete rejected consumption of",
                path: claimed_path,
                source,
            })?;
            continue;
        }
        let expired = signal.is_expired_at(now_unix_millis);
        if expired {
            publish_focus_ack(
                project_root.as_ref(),
                &signal,
                HubEditorFocusAckDispositionV1::RejectedExpired,
            )?;
        }
        fs::remove_file(&claimed_path).map_err(|source| HubFocusSignalError::Io {
            operation: "complete consumption of",
            path: claimed_path,
            source,
        })?;
        if !expired {
            consumed.push(signal);
        }
    }
    Ok(consumed)
}

pub fn focus_signal_path(
    project_root: impl AsRef<Path>,
    instance_id: &str,
    sequence: u64,
    request_id: HubSessionToken,
) -> Result<PathBuf, HubFocusSignalError> {
    hub_editor_focus_signal_path(project_root, instance_id, sequence, request_id)
        .map_err(Into::into)
}

pub fn publish_focus_ack(
    project_root: impl AsRef<Path>,
    request: &HubEditorFocusSignalV1,
    disposition: HubEditorFocusAckDispositionV1,
) -> Result<(), HubFocusSignalError> {
    let path = hub_editor_focus_ack_path(
        project_root,
        &request.target_instance_id,
        request.sequence,
        request.request_id,
    )?;
    let acknowledgement = HubEditorFocusAckV1::from_request(request, disposition);
    let bytes =
        serde_json::to_vec(&acknowledgement).map_err(|source| HubFocusSignalError::Encode {
            path: path.clone(),
            source,
        })?;
    atomic_write(&path, &bytes).map_err(|source| HubFocusSignalError::Io {
        operation: "publish acknowledgement",
        path,
        source,
    })
}

fn request_paths_in_sequence(directory: &Path) -> Result<Vec<PathBuf>, HubFocusSignalError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(HubFocusSignalError::Io {
                operation: "read",
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn unix_millis_now() -> Result<u64, HubFocusSignalError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .map_err(|source| HubFocusSignalError::Clock { source })
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
