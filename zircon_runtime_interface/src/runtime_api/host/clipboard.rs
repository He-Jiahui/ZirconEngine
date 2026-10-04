use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::dispatch::{UiClipboardRequest, UiClipboardTransferId, UiClipboardTransferOutcome};
use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 运行时送往平台宿主的剪贴板事务，携带视口和 UI surface 归属。
pub struct ZrRuntimeClipboardHostRequestV1 {
    pub target_viewport: ZrRuntimeViewportHandle,
    pub target_surface: u32,
    pub request: UiClipboardRequest,
}

impl ZrRuntimeClipboardHostRequestV1 {
    pub fn new(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        request: UiClipboardRequest,
    ) -> Self {
        Self {
            target_viewport,
            target_surface,
            request,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 宿主完成事务后回送的结果；运行时按 surface、传输 ID 和节点 owner 关联原请求。
pub struct ZrRuntimeClipboardResultV1 {
    pub target_surface: u32,
    pub transfer_id: UiClipboardTransferId,
    pub owner: UiNodeId,
    pub outcome: UiClipboardTransferOutcome,
}

impl ZrRuntimeClipboardResultV1 {
    pub fn new(
        target_surface: u32,
        transfer_id: UiClipboardTransferId,
        owner: UiNodeId,
        outcome: UiClipboardTransferOutcome,
    ) -> Self {
        Self {
            target_surface,
            transfer_id,
            owner,
            outcome,
        }
    }
}

#[cfg(test)]
#[path = "tests/clipboard.rs"]
mod tests;
