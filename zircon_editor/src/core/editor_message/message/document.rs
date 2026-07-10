use serde::{Deserialize, Serialize};

use crate::core::editor_message::DocumentId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentMessage {
    Opened { doc: DocumentId },
    Closed { doc: DocumentId },
    Saved { doc: DocumentId },
    DirtyChanged { doc: DocumentId, dirty: bool },
    FocusRequested { doc: DocumentId },
}
