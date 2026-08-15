use serde::{Deserialize, Serialize};

use crate::core::editor_message::{EditorMessageDelivery, EditorSubscriberId, EditorTopic};

use super::{EditorMessage, EditorMessageProtocol};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorMessageRequest {
    target: EditorSubscriberId,
    delivery: EditorMessageDelivery,
}

impl EditorMessageRequest {
    pub fn new(target: EditorSubscriberId, topic: EditorTopic, message: EditorMessage) -> Self {
        Self {
            target,
            delivery: EditorMessageDelivery::new(EditorMessageProtocol::Request, topic, message),
        }
    }

    pub(in crate::core::editor_message) fn from_delivery(
        target: EditorSubscriberId,
        delivery: EditorMessageDelivery,
    ) -> Self {
        debug_assert_eq!(delivery.protocol(), EditorMessageProtocol::Request);
        Self { target, delivery }
    }

    pub fn target(&self) -> EditorSubscriberId {
        self.target
    }

    pub fn topic(&self) -> &EditorTopic {
        self.delivery.topic()
    }

    pub fn message(&self) -> &EditorMessage {
        self.delivery.message()
    }

    pub(crate) fn shares_payload_with(&self, delivery: &EditorMessageDelivery) -> bool {
        self.delivery.shares_payload_with(delivery)
    }
}
