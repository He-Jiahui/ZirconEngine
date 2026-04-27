use crate::core::editor_event::{
    EditorAssetEvent, EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord,
    EditorEventResult, EditorEventSequence, EditorEventSource, EditorEventUndoPolicy,
};

pub(super) fn record_with_event_and_effects(
    event: EditorEvent,
    effects: Vec<EditorEventEffect>,
) -> EditorEventRecord {
    EditorEventRecord {
        event_id: EditorEventId::new(1),
        sequence: EditorEventSequence::new(1),
        source: EditorEventSource::Slint,
        event,
        operation_id: None,
        operation_display_name: None,
        effects,
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: 0,
        after_revision: 1,
        result: EditorEventResult::success(serde_json::json!({
            "revision": 1,
            "changed": true,
        })),
    }
}

pub(super) fn record_with_effects(effects: Vec<EditorEventEffect>) -> EditorEventRecord {
    record_with_event_and_effects(
        EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
            query: "cube".to_string(),
        }),
        effects,
    )
}
