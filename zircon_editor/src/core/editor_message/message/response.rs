use serde::{Deserialize, Serialize};

use super::EditorMessage;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
