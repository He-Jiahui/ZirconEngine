use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic, FocusMessage,
    HistoryContextId, ModeMessage, PlayStateKind, SceneModeId, SelectionDomain, TransactionMessage,
};

pub(super) fn topic(value: &str) -> EditorTopic {
    EditorTopic::parse(value).unwrap()
}

pub(super) fn view(value: &str) -> ViewInstanceId {
    ViewInstanceId::new(value)
}

pub(super) fn typed_messages() -> Vec<(&'static str, EditorMessage)> {
    vec![
        (
            "editor.document",
            EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Opened {
                doc: DocumentId::new(7),
            })),
        ),
        (
            "editor.transaction",
            EditorMessage::new(EditorMessagePayload::Transaction(
                TransactionMessage::Committed {
                    history: HistoryContextId::new(11),
                    label: "Move entity".to_string(),
                },
            )),
        ),
        (
            "editor.mode",
            EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
                from: PlayStateKind::Edit,
                to: PlayStateKind::Playing,
            })),
        ),
        (
            "editor.focus",
            EditorMessage::new(EditorMessagePayload::Focus(
                FocusMessage::SelectionChanged {
                    domain: SelectionDomain::Scene,
                    revision: 13,
                },
            )),
        ),
    ]
}

pub(super) fn response_message() -> EditorMessage {
    EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::SceneModeChanged {
        mode: SceneModeId::new("select"),
    }))
}
