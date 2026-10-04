use super::*;
use crate::state::{HubMessage, HubMessageId, ShellMessageId};

#[test]
fn archived_string_deserializes_into_raw_text_branch() {
    let message: HubMessage = serde_json::from_str("\"old detail\"").unwrap();

    assert_eq!(message, "old detail");
}

#[test]
fn unknown_id_degrades_to_raw_text_instead_of_failing_file_load() {
    let message: HubMessage =
        serde_json::from_str(r#"{"id":"future.message","params":["a","b"]}"#).unwrap();

    assert_eq!(message, "future.message: a, b");
}

#[test]
fn structured_message_renders_template_parameters() {
    let message = HubMessage::with_params(
        HubMessageId::Shell(ShellMessageId::OpenedPath),
        ["C:/Projects/Game"],
    );

    assert_eq!(
        message.render(HubLanguage::Chinese),
        "已打开 C:/Projects/Game"
    );
}
