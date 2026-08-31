use serde::{Deserialize, Serialize};

use super::{HubProtocolVersionV1, HubSessionToken};

/// A sequenced, deadline-bounded request asking an already-running editor instance to take window
/// attention.
///
/// The target is the instance identity from the authoritative project session lock. This DTO
/// deliberately contains no process-liveness or recovery data; those remain owned by the
/// editor's `SessionGuard` contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubEditorFocusSignalV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub request_id: HubSessionToken,
    pub target_instance_id: String,
    pub target_session_generation: u64,
    pub sequence: u64,
    pub deadline_unix_millis: u64,
}

impl HubEditorFocusSignalV1 {
    pub fn new(
        request_id: HubSessionToken,
        target_instance_id: impl Into<String>,
        target_session_generation: u64,
        sequence: u64,
        deadline_unix_millis: u64,
    ) -> Result<Self, HubEditorFocusSignalError> {
        let signal = Self {
            protocol_version: HubProtocolVersionV1,
            request_id,
            target_instance_id: target_instance_id.into(),
            target_session_generation,
            sequence,
            deadline_unix_millis,
        };
        signal.validate()?;
        Ok(signal)
    }

    pub fn validate(&self) -> Result<(), HubEditorFocusSignalError> {
        if self.target_instance_id.is_empty()
            || self.target_instance_id.len() > MAX_TARGET_INSTANCE_ID_BYTES
        {
            return Err(HubEditorFocusSignalError::InvalidTargetInstanceId);
        }
        if self.target_session_generation == 0 {
            return Err(HubEditorFocusSignalError::ZeroTargetSessionGeneration);
        }
        if self.sequence == 0 {
            return Err(HubEditorFocusSignalError::ZeroSequence);
        }
        if self.deadline_unix_millis == 0 {
            return Err(HubEditorFocusSignalError::ZeroDeadline);
        }
        Ok(())
    }

    pub const fn is_expired_at(&self, now_unix_millis: u64) -> bool {
        now_unix_millis >= self.deadline_unix_millis
    }
}

const MAX_TARGET_INSTANCE_ID_BYTES: usize = 128;

/// Categorical validation error for a malformed focus request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HubEditorFocusSignalError {
    InvalidTargetInstanceId,
    ZeroTargetSessionGeneration,
    ZeroSequence,
    ZeroDeadline,
}

impl std::fmt::Display for HubEditorFocusSignalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidTargetInstanceId => {
                "focus target instance id must be non-empty and bounded"
            }
            Self::ZeroTargetSessionGeneration => "focus target session generation must be non-zero",
            Self::ZeroSequence => "focus request sequence must be non-zero",
            Self::ZeroDeadline => "focus request deadline must be non-zero",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for HubEditorFocusSignalError {}
