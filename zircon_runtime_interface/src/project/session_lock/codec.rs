use std::collections::BTreeMap;

use super::{ProjectSessionLockRecordDecodeError, ProjectSessionLockRecordV1};

const PROJECT_SESSION_LOCK_VERSION_V1: u32 = 1;

pub fn encode_project_session_lock_record(record: &ProjectSessionLockRecordV1) -> String {
    format!(
        "version={PROJECT_SESSION_LOCK_VERSION_V1}\\nprocess_id={}\\ninstance_id={}\\nheartbeat_unix_millis={}\\n",
        record.process_id(),
        record.instance_id(),
        record.heartbeat_unix_millis()
    )
}

pub fn decode_project_session_lock_record(
    source: &str,
) -> Result<ProjectSessionLockRecordV1, ProjectSessionLockRecordDecodeError> {
    let mut values = BTreeMap::new();
    for line in source.lines() {
        let (key, value) = line.split_once('=').ok_or_else(|| {
            ProjectSessionLockRecordDecodeError::new("every line must be key=value")
        })?;
        if values.insert(key, value).is_some() {
            return Err(ProjectSessionLockRecordDecodeError::new(format!(
                "duplicate field `{key}`"
            )));
        }
    }
    let version = values
        .remove("version")
        .ok_or_else(|| invalid_field("version"))?;
    if version != PROJECT_SESSION_LOCK_VERSION_V1.to_string() {
        return Err(ProjectSessionLockRecordDecodeError::new(
            "unsupported version",
        ));
    }
    let process_id = parse_number(values.remove("process_id"), "process_id")?;
    let instance_id = values
        .remove("instance_id")
        .ok_or_else(|| invalid_field("instance_id"))?;
    let heartbeat_unix_millis = parse_number(
        values.remove("heartbeat_unix_millis"),
        "heartbeat_unix_millis",
    )?;
    if !values.is_empty() {
        return Err(ProjectSessionLockRecordDecodeError::new("unknown field"));
    }
    ProjectSessionLockRecordV1::new(process_id, instance_id, heartbeat_unix_millis)
}

fn parse_number<T>(
    value: Option<&str>,
    field: &'static str,
) -> Result<T, ProjectSessionLockRecordDecodeError>
where
    T: std::str::FromStr,
{
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| invalid_field(field))
}

fn invalid_field(field: &'static str) -> ProjectSessionLockRecordDecodeError {
    ProjectSessionLockRecordDecodeError::new(format!("invalid or missing {field}"))
}
