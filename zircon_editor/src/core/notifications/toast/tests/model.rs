use super::{ToastNotification, MAX_TOAST_KEY_BYTES};

#[test]
fn bounded_message_preserves_utf8_boundary_and_contract_limit() {
    let message = ToastNotification::bounded_message(&"中".repeat(MAX_TOAST_KEY_BYTES), "fallback");

    assert!(message.len() <= MAX_TOAST_KEY_BYTES);
    assert!(message.is_char_boundary(message.len()));
    assert!(message.ends_with("..."));
}
