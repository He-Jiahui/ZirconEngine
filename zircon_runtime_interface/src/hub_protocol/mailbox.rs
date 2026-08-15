use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{HubEditorLaunchOutcomeV1, HubProtocolVersionV1};

/// File-mailbox payload shared by Hub and Editor for a single launch session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubEditorMailboxV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub outcome: HubEditorLaunchOutcomeV1,
}

impl HubEditorMailboxV1 {
    pub fn ready(pid: u32, project: impl Into<PathBuf>) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            outcome: HubEditorLaunchOutcomeV1::Ready {
                pid,
                project: project.into(),
            },
        }
    }

    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            outcome: HubEditorLaunchOutcomeV1::Failed {
                reason: reason.into(),
            },
        }
    }
}
