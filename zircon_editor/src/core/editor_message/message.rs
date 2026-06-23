use serde::{Deserialize, Serialize};

use crate::core::editor_event::ViewInstanceId;

use super::{EditorSubscriberId, EditorTopic, EditorViewInvalidationMask};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMessagePayload {
    Empty,
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorViewDirtyMark {
    view: ViewInstanceId,
    mask: EditorViewInvalidationMask,
}

impl EditorViewDirtyMark {
    pub fn new(view: ViewInstanceId, mask: EditorViewInvalidationMask) -> Self {
        Self { view, mask }
    }

    pub fn view(&self) -> &ViewInstanceId {
        &self.view
    }

    pub fn mask(&self) -> EditorViewInvalidationMask {
        self.mask
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMessage {
    payload: EditorMessagePayload,
    dirty: Option<EditorViewDirtyMark>,
}

impl EditorMessage {
    pub fn empty() -> Self {
        Self {
            payload: EditorMessagePayload::Empty,
            dirty: None,
        }
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self {
            payload: EditorMessagePayload::Text(value.into()),
            dirty: None,
        }
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMessageProtocol {
    Publish,
    Request,
    Broadcast,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMessageDelivery {
    protocol: EditorMessageProtocol,
    topic: EditorTopic,
    message: EditorMessage,
}

impl EditorMessageDelivery {
    pub fn new(
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> Self {
        Self {
            protocol,
            topic,
            message,
        }
    }

    pub fn protocol(&self) -> EditorMessageProtocol {
        self.protocol
    }

    pub fn topic(&self) -> &EditorTopic {
        &self.topic
    }

    pub fn message(&self) -> &EditorMessage {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMessageRequest {
    target: EditorSubscriberId,
    topic: EditorTopic,
    message: EditorMessage,
}

impl EditorMessageRequest {
    pub fn new(target: EditorSubscriberId, topic: EditorTopic, message: EditorMessage) -> Self {
        Self {
            target,
            topic,
            message,
        }
    }

    pub fn target(&self) -> EditorSubscriberId {
        self.target
    }

    pub fn topic(&self) -> &EditorTopic {
        &self.topic
    }

    pub fn message(&self) -> &EditorMessage {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMessageResponse {
    message: EditorMessage,
}

impl EditorMessageResponse {
    pub fn handled(message: EditorMessage) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &EditorMessage {
        &self.message
    }
}
