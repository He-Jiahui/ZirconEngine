use serde::{Deserialize, Serialize};

use crate::core::editor_message::{EditorSubscriberId, EditorTopic};

use super::EditorMessage;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
