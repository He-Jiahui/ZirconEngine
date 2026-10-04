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
