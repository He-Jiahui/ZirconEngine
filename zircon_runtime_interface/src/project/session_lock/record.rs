use super::ProjectSessionLockRecordDecodeError;

/// Persistent identity of the editor instance holding a project session lease.
///
/// This record is recovery metadata only. It is not itself a liveness claim; hosts must consult
/// their platform lease before treating it as an active owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionLockRecordV1 {
    process_id: u32,
    instance_id: String,
    heartbeat_unix_millis: u64,
}

impl ProjectSessionLockRecordV1 {
    pub fn new(
        process_id: u32,
        instance_id: impl Into<String>,
        heartbeat_unix_millis: u64,
    ) -> Result<Self, ProjectSessionLockRecordDecodeError> {
        let instance_id = instance_id.into();
        validate_instance_id(&instance_id)?;
        Ok(Self {
            process_id,
            instance_id,
            heartbeat_unix_millis,
        })
    }

    pub const fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub const fn heartbeat_unix_millis(&self) -> u64 {
        self.heartbeat_unix_millis
    }

    pub fn with_heartbeat_unix_millis(&self, heartbeat_unix_millis: u64) -> Self {
        Self {
            process_id: self.process_id,
            instance_id: self.instance_id.clone(),
            heartbeat_unix_millis,
        }
    }
}

pub(super) fn validate_instance_id(
    instance_id: &str,
) -> Result<(), ProjectSessionLockRecordDecodeError> {
    if instance_id.is_empty()
        || !instance_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ProjectSessionLockRecordDecodeError::new(
            "invalid or missing instance_id",
        ));
    }
    Ok(())
}
