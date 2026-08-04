use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::SessionGuardError;

pub const SESSION_LOCK_FILE_NAME: &str = "session.lock";
pub(super) const SESSION_LOCK_DIRECTORY: &str = ".zircon";
const SESSION_LOCK_VERSION: u32 = 1;

/// Persistent process identity and heartbeat retained after an unclean editor exit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionLockRecord {
    process_id: u32,
    instance_id: String,
    heartbeat_unix_millis: u64,
}

impl SessionLockRecord {
    pub const fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub const fn heartbeat_unix_millis(&self) -> u64 {
        self.heartbeat_unix_millis
    }
}

/// The recovery layer never treats a residual lock as permission to remove it implicitly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionLockInspection {
    Missing,
    Residual(SessionLockRecord),
}

pub(super) fn session_lock_path(project_root: &Path) -> PathBuf {
    session_lock_directory(project_root).join(SESSION_LOCK_FILE_NAME)
}

pub(super) fn session_lock_directory(project_root: &Path) -> PathBuf {
    project_root.join(SESSION_LOCK_DIRECTORY)
}

pub(super) fn new_record(now: SystemTime) -> Result<SessionLockRecord, SessionGuardError> {
    static NEXT_INSTANCE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    let process_id = std::process::id();
    let heartbeat_unix_millis = unix_millis(now)?;
    let sequence = NEXT_INSTANCE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(SessionLockRecord {
        process_id,
        instance_id: format!("{process_id}-{heartbeat_unix_millis}-{sequence}"),
        heartbeat_unix_millis,
    })
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

pub(super) fn read_lock(path: &Path) -> Result<Option<SessionLockRecord>, SessionGuardError> {
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
    parse_record(path, &source).map(Some)
}

pub(super) fn encode_record(record: &SessionLockRecord) -> String {
    format!(
        "version={SESSION_LOCK_VERSION}\nprocess_id={}\ninstance_id={}\nheartbeat_unix_millis={}\n",
        record.process_id, record.instance_id, record.heartbeat_unix_millis
    )
}

fn parse_record(path: &Path, source: &str) -> Result<SessionLockRecord, SessionGuardError> {
    let mut values = BTreeMap::new();
    for line in source.lines() {
        let (key, value) =
            line.split_once('=')
                .ok_or_else(|| SessionGuardError::InvalidRecord {
                    path: path.to_path_buf(),
                    message: "every line must be key=value".to_string(),
                })?;
        if values.insert(key, value).is_some() {
            return Err(SessionGuardError::InvalidRecord {
                path: path.to_path_buf(),
                message: format!("duplicate field `{key}`"),
            });
        }
    }
    let version = values
        .remove("version")
        .ok_or_else(|| invalid_field(path, "version"))?;
    if version != SESSION_LOCK_VERSION.to_string() {
        return Err(SessionGuardError::InvalidRecord {
            path: path.to_path_buf(),
            message: "unsupported version".to_string(),
        });
    }
    let process_id = parse_number(path, values.remove("process_id"), "process_id")?;
    let instance_id = values
        .remove("instance_id")
        .filter(|value| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || byte == b'-')
        })
        .ok_or_else(|| invalid_field(path, "instance_id"))?
        .to_string();
    let heartbeat_unix_millis = parse_number(
        path,
        values.remove("heartbeat_unix_millis"),
        "heartbeat_unix_millis",
    )?;
    if !values.is_empty() {
        return Err(SessionGuardError::InvalidRecord {
            path: path.to_path_buf(),
            message: "unknown field".to_string(),
        });
    }
    Ok(SessionLockRecord {
        process_id,
        instance_id,
        heartbeat_unix_millis,
    })
}

fn parse_number<T>(
    path: &Path,
    value: Option<&str>,
    field: &'static str,
) -> Result<T, SessionGuardError>
where
    T: std::str::FromStr,
{
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| invalid_field(path, field))
}

fn invalid_field(path: &Path, field: &'static str) -> SessionGuardError {
    SessionGuardError::InvalidRecord {
        path: path.to_path_buf(),
        message: format!("invalid or missing {field}"),
    }
}
