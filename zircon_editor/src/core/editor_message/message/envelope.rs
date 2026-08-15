use serde::{Deserialize, Serialize};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;

use super::{EditorMessagePayload, EditorViewDirtyMark};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorMessage {
    payload: EditorMessagePayload,
    dirty: Option<EditorViewDirtyMark>,
}

impl EditorMessage {
    pub fn new(payload: EditorMessagePayload) -> Self {
        Self {
            payload,
            dirty: None,
        }
    }

    pub fn custom(schema_id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self::new(EditorMessagePayload::Custom {
            schema_id: schema_id.into(),
            payload,
        })
    }

    pub fn with_dirty(mut self, view: ViewInstanceId, mask: EditorViewInvalidationMask) -> Self {
        self.dirty = Some(EditorViewDirtyMark::new(view, mask));
        self
    }

    pub fn payload(&self) -> &EditorMessagePayload {
        &self.payload
    }

    pub fn dirty(&self) -> Option<&EditorViewDirtyMark> {
        self.dirty.as_ref()
    }

    pub(in crate::core::editor_message) fn coalesce_latest_from(mut self, previous: &Self) -> Self {
        if let (
            EditorMessagePayload::SceneInspection(current),
            EditorMessagePayload::SceneInspection(previous),
        ) = (&mut self.payload, &previous.payload)
        {
            current.coalesce_selection_from(previous);
        }
        self
    }
}
