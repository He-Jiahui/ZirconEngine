use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    HubEditorLaunchOutcomeV1, HubEditorReadyReceiptV1, HubEditorStartupFailureCodeV1,
    HubProtocolVersionV1, HubSessionToken,
};

/// File-mailbox payload shared by Hub and Editor for a single launch session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubEditorMailboxV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub launch_session: HubSessionToken,
    pub outcome: HubEditorLaunchOutcomeV1,
}

impl HubEditorMailboxV1 {
    pub fn ready(launch_session: HubSessionToken, receipt: HubEditorReadyReceiptV1) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            launch_session,
            outcome: HubEditorLaunchOutcomeV1::Ready { receipt },
        }
    }

    pub fn failed(launch_session: HubSessionToken, code: HubEditorStartupFailureCodeV1) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            launch_session,
            outcome: HubEditorLaunchOutcomeV1::Failed { code },
        }
    }

    pub fn ready_receipt(&self) -> Option<&HubEditorReadyReceiptV1> {
        match &self.outcome {
            HubEditorLaunchOutcomeV1::Ready { receipt } => Some(receipt),
            HubEditorLaunchOutcomeV1::Failed { .. } => None,
        }
    }

    pub fn validate_launch_session(
        &self,
        expected: HubSessionToken,
    ) -> Result<(), HubEditorMailboxSessionError> {
        if self.launch_session == expected {
            Ok(())
        } else {
            Err(HubEditorMailboxSessionError)
        }
    }
}

/// Opaque mismatch result for a mailbox not bound to the launch being supervised.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HubEditorMailboxSessionError;

impl fmt::Display for HubEditorMailboxSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Hub Editor mailbox launch session does not match the supervised child")
    }
}

impl std::error::Error for HubEditorMailboxSessionError {}
