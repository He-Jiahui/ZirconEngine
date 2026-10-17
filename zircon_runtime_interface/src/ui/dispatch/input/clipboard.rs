use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::UiNodeId;

use super::UiInputEventMetadata;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
/// 剪贴板传输的 UUID 标识；默认值为 nil，调用新建入口后才获得有效随机标识。
pub struct UiClipboardTransferId(Uuid);

impl UiClipboardTransferId {
    /// 生成新的 UUID v4 传输标识。
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    /// 返回标识是否非 nil；不代表对应请求仍处于待处理状态。
    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 表示剪贴板操作意图；Runtime 将 Copy/Cut 转为写文本请求，将 Paste 转为读文本请求。
pub enum UiClipboardTransferIntent {
    #[default]
    Copy,
    Cut,
    Paste,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 请求宿主执行的剪贴板方向；与 intent 的组合由 Runtime 检查。
pub enum UiClipboardRequestKind {
    ReadText,
    WriteText,
}

/// 发往宿主的异步剪贴板请求；完成事件须匹配传输 ID、owner 和发起时的编辑版本，才能修改当前文本。
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
/// 宿主报告的传输失败原因；它不同于 Runtime 对回执身份、策略或结果的拒绝状态。
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
/// 宿主对一次读写请求的完成结果；读回携带文本，写入完成不携带文本，失败携带原因。
pub enum UiClipboardTransferOutcome {
    ReadText { text: String },
    WriteText,
    Failed { reason: UiClipboardTransferFailure },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 宿主返回给 Runtime 的剪贴板完成事件，携带原传输标识、owner 与操作结果。
pub struct UiClipboardInputEvent {
    pub metadata: UiInputEventMetadata,
    pub transfer_id: UiClipboardTransferId,
    pub owner: UiNodeId,
    pub outcome: UiClipboardTransferOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Runtime 对剪贴板传输回执的归类，区分应用、失败及身份、时效、策略或结果拒绝。
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
/// 输入派发结果中的剪贴板传输摘要，不包含传输正文。
pub struct UiClipboardTransferReceipt {
    pub transfer_id: UiClipboardTransferId,
    pub intent: Option<UiClipboardTransferIntent>,
    pub status: UiClipboardTransferStatus,
}
