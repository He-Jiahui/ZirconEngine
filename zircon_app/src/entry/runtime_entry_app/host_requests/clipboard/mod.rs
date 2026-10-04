//! 动态 Runtime 剪贴板请求的宿主完成与类型化结果回送边界。
//! 平台动作在目标窗口线程完成后才回送结果；请求失败仍应产生明确失败回执。

mod platform;

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::ui::dispatch::{
    UiClipboardRequest, UiClipboardRequestKind, UiClipboardTransferFailure,
    UiClipboardTransferIntent, UiClipboardTransferOutcome,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeClipboardHostRequestV1, ZrRuntimeClipboardResultV1, ZrRuntimeEventV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1,
    ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1,
};

use super::super::RuntimeEntryApp;

trait ClipboardOperations {
    fn read_text(&mut self) -> Result<String, UiClipboardTransferFailure>;
    fn write_text(&mut self, text: &str) -> Result<(), UiClipboardTransferFailure>;
}

struct PlatformClipboard<'window> {
    window: Option<&'window dyn winit::window::Window>,
}

impl ClipboardOperations for PlatformClipboard<'_> {
    fn read_text(&mut self) -> Result<String, UiClipboardTransferFailure> {
        platform::read_text(self.window)
    }

    fn write_text(&mut self, text: &str) -> Result<(), UiClipboardTransferFailure> {
        platform::write_text(self.window, text)
    }
}

/// 针对目标视口执行平台剪贴板动作，再把同一 transfer 的完成回执同步交还 Runtime。
pub(super) fn apply_runtime_clipboard_host_request(
    app: &mut RuntimeEntryApp,
    event_loop: &dyn ActiveEventLoop,
    request: ZrRuntimeClipboardHostRequestV1,
) -> Result<(), String> {
    let mut clipboard = PlatformClipboard {
        window: app.window.as_deref(),
    };
    let outcome = if request.target_viewport == app.viewport {
        complete_clipboard_request(&request.request, &mut clipboard)
    } else {
        UiClipboardTransferOutcome::Failed {
            reason: UiClipboardTransferFailure::HostDisconnected,
        }
    };
    let result = ZrRuntimeClipboardResultV1::new(
        request.target_surface,
        request.request.transfer_id,
        request.request.owner,
        outcome,
    );
    let payload = serde_json::to_vec(&result).map_err(|error| error.to_string())?;
    if payload.len() > ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1.max_encoded_bytes {
        return Err("runtime clipboard result exceeds the event payload limit".to_string());
    }
    let event = ZrRuntimeEventV1::clipboard_result(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        app.viewport,
        ZrByteSlice {
            data: payload.as_ptr(),
            len: payload.len(),
        },
    );
    if app.dispatch_runtime_event(event_loop, event) {
        Ok(())
    } else {
        Err("runtime rejected the clipboard result event".to_string())
    }
}

fn complete_clipboard_request(
    request: &UiClipboardRequest,
    clipboard: &mut impl ClipboardOperations,
) -> UiClipboardTransferOutcome {
    let result = match (request.intent, request.kind, request.text.as_deref()) {
        (UiClipboardTransferIntent::Paste, UiClipboardRequestKind::ReadText, None) => clipboard
            .read_text()
            .and_then(|text| {
                (text.len() <= ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1)
                    .then_some(text)
                    .ok_or(UiClipboardTransferFailure::PayloadTooLarge)
            })
            .map(|text| UiClipboardTransferOutcome::ReadText { text }),
        (
            UiClipboardTransferIntent::Copy | UiClipboardTransferIntent::Cut,
            UiClipboardRequestKind::WriteText,
            Some(text),
        ) if text.len() <= ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 => clipboard
            .write_text(text)
            .map(|()| UiClipboardTransferOutcome::WriteText),
        (_, UiClipboardRequestKind::WriteText, Some(text))
            if text.len() > ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 =>
        {
            Err(UiClipboardTransferFailure::PayloadTooLarge)
        }
        _ => Err(UiClipboardTransferFailure::Unknown),
    };
    result.unwrap_or_else(|reason| UiClipboardTransferOutcome::Failed { reason })
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
