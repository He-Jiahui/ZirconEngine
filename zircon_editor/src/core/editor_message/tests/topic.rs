use super::EditorTopic;

#[test]
fn built_in_transaction_topic_is_canonical_and_valid() {
    assert_eq!(EditorTopic::transaction().as_str(), "editor.transaction");
}

#[test]
fn built_in_document_topic_is_canonical_and_valid() {
    assert_eq!(EditorTopic::document().as_str(), "editor.document");
}

#[test]
fn built_in_log_topic_is_canonical_and_valid() {
    assert_eq!(EditorTopic::log().as_str(), "editor.log");
}

#[test]
fn built_in_i18n_topic_is_canonical_and_valid() {
    assert_eq!(EditorTopic::i18n().as_str(), "editor.i18n");
}

#[test]
fn built_in_tool_topic_is_canonical_and_valid() {
    assert_eq!(EditorTopic::tool().as_str(), "editor.tool");
}
