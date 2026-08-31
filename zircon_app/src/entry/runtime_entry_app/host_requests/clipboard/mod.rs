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
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::dispatch::UiClipboardTransferId;
    use zircon_runtime_interface::ui::event_ui::UiNodeId;

    #[derive(Default)]
    struct FakeClipboard {
        read: Option<Result<String, UiClipboardTransferFailure>>,
        writes: Vec<String>,
    }

    impl ClipboardOperations for FakeClipboard {
        fn read_text(&mut self) -> Result<String, UiClipboardTransferFailure> {
            self.read
                .take()
                .unwrap_or(Err(UiClipboardTransferFailure::ContentUnavailable))
        }

        fn write_text(&mut self, text: &str) -> Result<(), UiClipboardTransferFailure> {
            self.writes.push(text.to_string());
            Ok(())
        }
    }

    fn request(
        intent: UiClipboardTransferIntent,
        kind: UiClipboardRequestKind,
        text: Option<String>,
    ) -> UiClipboardRequest {
        UiClipboardRequest {
            transfer_id: UiClipboardTransferId::issue(),
            intent,
            expected_edit_revision: 3,
            kind,
            owner: UiNodeId::new(9),
            text,
        }
    }

    #[test]
    fn typed_copy_and_paste_complete_only_after_backend_success() {
        let mut clipboard = FakeClipboard {
            read: Some(Ok("paste".to_string())),
            ..FakeClipboard::default()
        };
        assert_eq!(
            complete_clipboard_request(
                &request(
                    UiClipboardTransferIntent::Copy,
                    UiClipboardRequestKind::WriteText,
                    Some("copy".to_string()),
                ),
                &mut clipboard,
            ),
            UiClipboardTransferOutcome::WriteText
        );
        assert_eq!(clipboard.writes, ["copy"]);
        assert_eq!(
            complete_clipboard_request(
                &request(
                    UiClipboardTransferIntent::Paste,
                    UiClipboardRequestKind::ReadText,
                    None,
                ),
                &mut clipboard,
            ),
            UiClipboardTransferOutcome::ReadText {
                text: "paste".to_string()
            }
        );
    }

    #[test]
    fn malformed_and_oversized_requests_fail_without_backend_write() {
        let mut clipboard = FakeClipboard::default();
        let malformed = request(
            UiClipboardTransferIntent::Paste,
            UiClipboardRequestKind::WriteText,
            Some("wrong".to_string()),
        );
        assert_eq!(
            complete_clipboard_request(&malformed, &mut clipboard),
            UiClipboardTransferOutcome::Failed {
                reason: UiClipboardTransferFailure::Unknown
            }
        );
        let oversized = request(
            UiClipboardTransferIntent::Copy,
            UiClipboardRequestKind::WriteText,
            Some("x".repeat(ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 + 1)),
        );
        assert_eq!(
            complete_clipboard_request(&oversized, &mut clipboard),
            UiClipboardTransferOutcome::Failed {
                reason: UiClipboardTransferFailure::PayloadTooLarge
            }
        );
        assert!(clipboard.writes.is_empty());
    }
}
