use serde::{Deserialize, Serialize};

use crate::core::editor_message::EditorTopic;

use super::{EditorMessage, EditorMessageProtocol};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
