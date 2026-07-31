use crate::core::editing::engine::{HistoryContextId, TransactionId};
use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic, FocusMessage,
    ModeMessage, PlayStateKind, SceneInspectionFieldsDelta, SceneInspectionMessage,
    SceneInspectionPropertyPath, SceneModeId, SelectionDomain, TransactionMessage,
    TOPIC_SCENE_INSPECTION,
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
                    transaction: TransactionId::from_sequence(1),
                    history: HistoryContextId::Document(DocumentId::new(11)),
                    label: "Move entity".to_string(),
                    timestamp_frame: 0,
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
        (
            TOPIC_SCENE_INSPECTION,
            EditorMessage::new(EditorMessagePayload::SceneInspection(
                SceneInspectionMessage::delta(
                    12,
                    13,
                    Some(42),
                    vec![40],
                    vec![42],
                    vec![7],
                    SceneInspectionFieldsDelta::delta(
                        42,
                        vec![SceneInspectionPropertyPath::new(
                            "zircon_runtime::scene::components::LocalTransform",
                            "translation",
                        )],
                        vec![SceneInspectionPropertyPath::new(
                            "zircon_runtime::scene::components::Name",
                            "value",
                        )],
                    ),
                ),
            )),
        ),
    ]
}

pub(super) fn response_message() -> EditorMessage {
    EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::SceneModeChanged {
        mode: SceneModeId::new("select"),
    }))
}
