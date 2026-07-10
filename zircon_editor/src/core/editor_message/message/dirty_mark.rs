use serde::{Deserialize, Serialize};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;

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
