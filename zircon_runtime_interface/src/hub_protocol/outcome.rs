use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Terminal result written by Editor after a Hub-initiated launch reaches a decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum HubEditorLaunchOutcomeV1 {
    Ready { pid: u32, project: PathBuf },
    Failed { reason: String },
}
