use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::session_lock::{
    PROJECT_SESSION_LOCK_FILE_NAME, ProjectSessionGenerationV1,
    decode_project_session_admission_record, encode_project_session_admission_record,
    project_session_lock_path,
};

use super::{SessionAdmissionRequest, SessionGuardError};

pub use zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionRecordV1;

pub const SESSION_LOCK_FILE_NAME: &str = PROJECT_SESSION_LOCK_FILE_NAME;
pub(super) const SESSION_LOCK_DIRECTORY: &str = ".zircon";

/// The recovery layer never treats a residual lock as permission to remove it implicitly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionLockInspection {
    Missing,
    Residual(ProjectSessionAdmissionRecordV1),
}

pub(super) fn session_lock_path(project_root: &Path) -> PathBuf {
    project_session_lock_path(project_root)
}

pub(super) fn session_lock_directory(project_root: &Path) -> PathBuf {
    project_root.join(SESSION_LOCK_DIRECTORY)
}

pub(super) fn new_record(
    admission: &SessionAdmissionRequest,
    now: SystemTime,
) -> Result<ProjectSessionAdmissionRecordV1, SessionGuardError> {
    static NEXT_INSTANCE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    let process_id = std::process::id();
    let heartbeat_unix_millis = unix_millis(now)?;
    let sequence = NEXT_INSTANCE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let record = ProjectSessionAdmissionRecordV1::claim(
        process_id,
        format!("{process_id}-{heartbeat_unix_millis}-{sequence}"),
        admission.principal(),
        admission.build_set_id().clone(),
        admission.operation_id(),
        heartbeat_unix_millis,
    )
    .expect("generated editor session instance identity must be valid");
    Ok(record)
}

pub(super) fn unix_millis(now: SystemTime) -> Result<u64, SessionGuardError> {
    let millis = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SessionGuardError::ClockBeforeUnixEpoch)?
        .as_millis();
    u64::try_from(millis).map_err(|_| SessionGuardError::ClockBeforeUnixEpoch)
}

pub(super) fn inspect_lock(path: &Path) -> Result<SessionLockInspection, SessionGuardError> {
    Ok(match read_lock(path)? {
        Some(record) => SessionLockInspection::Residual(record),
        None => SessionLockInspection::Missing,
    })
}

pub(super) fn read_lock(
    path: &Path,
) -> Result<Option<ProjectSessionAdmissionRecordV1>, SessionGuardError> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(SessionGuardError::Io {
                operation: "read session lock",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    decode_project_session_admission_record(&source)
        .map(Some)
        .map_err(|source| SessionGuardError::InvalidRecord {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

pub(super) fn encode_record(record: &ProjectSessionAdmissionRecordV1) -> String {
    encode_project_session_admission_record(record)
}

pub(super) fn next_session_generation() -> Result<ProjectSessionGenerationV1, SessionGuardError> {
    static NEXT_SESSION_GENERATION: AtomicU64 = AtomicU64::new(1);

    let value = NEXT_SESSION_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| SessionGuardError::SessionGenerationExhausted)?;
    ProjectSessionGenerationV1::new(value).ok_or(SessionGuardError::SessionGenerationExhausted)
}
