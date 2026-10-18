//! Runtime 到平台宿主的请求 ABI 及对应的完成结果。
//!
//! `ZrRuntimeHostRequestV1` 的六种变体组成动态会话 drain 的批次；UI 的 IME/clipboard 事务保留
//! 专用载体，通用 UI effect 则走 `UiHost`。同文件导出的 Gameplay 场景切换意图是 World resource，
//! 不属于该六变体批次。

#[path = "clipboard.rs"]
mod clipboard;
#[path = "host_requests.rs"]
mod host_requests;
#[path = "ui_action.rs"]
mod ui_action;
#[path = "ui_host_request.rs"]
mod ui_host_request;

pub use clipboard::{ZrRuntimeClipboardHostRequestV1, ZrRuntimeClipboardResultV1};
pub use host_requests::{
    ZrRuntimeCursorGrabModeV1, ZrRuntimeCursorHostRequestKindV1, ZrRuntimeCursorHostRequestV1,
    ZrRuntimeCursorPositionV1, ZrRuntimeGamepadRumbleRequestKindV1,
    ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeGamepadRumbleRequestV2, ZrRuntimeHostRequestBatchV1,
    ZrRuntimeHostRequestV1, ZrRuntimeImeCoordinateSpaceV1, ZrRuntimeImeCursorAreaV1,
    ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeHostRequestV1, ZrRuntimeImeSurroundingTextV1,
    ZrRuntimeImeTextRangeV1, ZrRuntimeProjectSceneTransitionPolicyV1,
    ZrRuntimeProjectSceneTransitionRequestErrorV1, ZrRuntimeProjectSceneTransitionRequestV1,
    ZrRuntimeProjectSceneTransitionResultV1, ZrRuntimeProjectSceneTransitionStatusV1,
};
pub use ui_action::ZrRuntimeUiActionHostRequestV1;
pub use ui_host_request::{ZrRuntimeUiHostRequestKindV1, ZrRuntimeUiHostRequestV1};
