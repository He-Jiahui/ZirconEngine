use serde::{Deserialize, Serialize};

use super::HubEditorReadyReceiptV1;

/// Terminal result written by Editor after a Hub-initiated launch reaches a decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum HubEditorLaunchOutcomeV1 {
    Ready { receipt: HubEditorReadyReceiptV1 },
    Failed { code: HubEditorStartupFailureCodeV1 },
}

/// Public, path-redacted category for a terminal Hub launch failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubEditorStartupFailureCodeV1 {
    Startup,
    ProjectActivation,
    FocusInboxBinding,
    NativeWindow,
    FirstPresent,
    HostWindow,
    MailboxPublish,
}
