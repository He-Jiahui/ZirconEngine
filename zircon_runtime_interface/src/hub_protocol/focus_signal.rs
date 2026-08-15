use serde::{Deserialize, Serialize};

use super::{HubProtocolVersionV1, HubSessionToken};

/// A one-shot request asking an already-running editor instance to take window attention.
///
/// The target is the instance identity from the authoritative project session lock. This DTO
/// deliberately contains no process-liveness or recovery data; those remain owned by the
/// editor's `SessionGuard` contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubEditorFocusSignalV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub session: HubSessionToken,
    pub target_instance_id: String,
}

impl HubEditorFocusSignalV1 {
    pub fn new(session: HubSessionToken, target_instance_id: impl Into<String>) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            session,
            target_instance_id: target_instance_id.into(),
        }
    }
}
