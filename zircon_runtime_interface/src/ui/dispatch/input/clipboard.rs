use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::UiNodeId;

use super::UiInputEventMetadata;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiClipboardTransferId(Uuid);

impl UiClipboardTransferId {
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiClipboardTransferIntent {
    #[default]
    Copy,
    Cut,
    Paste,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiClipboardRequestKind {
    ReadText,
    WriteText,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiClipboardRequest {
    #[serde(default)]
    pub transfer_id: UiClipboardTransferId,
    #[serde(default)]
    pub intent: UiClipboardTransferIntent,
    #[serde(default)]
    pub expected_edit_revision: u64,
    pub kind: UiClipboardRequestKind,
    pub owner: UiNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiClipboardTransferFailure {
    Unavailable,
    PermissionDenied,
    Unsupported,
    ContentUnavailable,
    PayloadTooLarge,
    Cancelled,
    Timeout,
    HostDisconnected,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum UiClipboardTransferOutcome {
    ReadText { text: String },
    WriteText,
    Failed { reason: UiClipboardTransferFailure },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiClipboardInputEvent {
    pub metadata: UiInputEventMetadata,
    pub transfer_id: UiClipboardTransferId,
    pub owner: UiNodeId,
    pub outcome: UiClipboardTransferOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiClipboardTransferStatus {
    Applied,
    Failed,
    RejectedUnknown,
    RejectedOwner,
    RejectedStale,
    RejectedOutcome,
    RejectedPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiClipboardTransferReceipt {
    pub transfer_id: UiClipboardTransferId,
    pub intent: Option<UiClipboardTransferIntent>,
    pub status: UiClipboardTransferStatus,
}
