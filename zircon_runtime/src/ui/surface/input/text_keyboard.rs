#[path = "text_keyboard/clipboard.rs"]
mod clipboard;
#[path = "text_keyboard/edit_actions.rs"]
mod edit_actions;
#[path = "text_keyboard/history.rs"]
mod history;
#[path = "text_keyboard/payload.rs"]
mod payload;

pub(super) use clipboard::{keyboard_clipboard_action, KeyboardClipboardAction};
pub(super) use edit_actions::keyboard_text_edit_actions;
pub(super) use history::keyboard_text_history_direction;
pub(super) use payload::{keyboard_requests_newline, keyboard_text_payload};
