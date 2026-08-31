use serde::{Deserialize, Serialize};

use super::{HubEditorFocusSignalV1, HubProtocolVersionV1, HubSessionToken};

/// Owner-authored terminal disposition for one sequenced Hub focus request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubEditorFocusAckDispositionV1 {
    Focused,
    RejectedExpired,
    RejectedTargetMismatch,
    RejectedInboxFull,
    RejectedStale,
}

/// Typed acknowledgement written only by the addressed Editor owner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubEditorFocusAckV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub request_id: HubSessionToken,
    pub target_instance_id: String,
    pub target_session_generation: u64,
    pub sequence: u64,
    pub disposition: HubEditorFocusAckDispositionV1,
}

impl HubEditorFocusAckV1 {
    pub fn focused(request: &HubEditorFocusSignalV1) -> Self {
        Self::from_request(request, HubEditorFocusAckDispositionV1::Focused)
    }

    pub fn from_request(
        request: &HubEditorFocusSignalV1,
        disposition: HubEditorFocusAckDispositionV1,
    ) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            request_id: request.request_id,
            target_instance_id: request.target_instance_id.clone(),
            target_session_generation: request.target_session_generation,
            sequence: request.sequence,
            disposition,
        }
    }

    pub fn matches_request(&self, request: &HubEditorFocusSignalV1) -> bool {
        self.request_id == request.request_id
            && self.target_instance_id == request.target_instance_id
            && self.target_session_generation == request.target_session_generation
            && self.sequence == request.sequence
    }
}
