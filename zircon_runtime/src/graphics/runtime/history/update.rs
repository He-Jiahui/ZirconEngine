use crate::graphics::visibility::VisibilityStaticIndex;
use crate::graphics::{FrameHistoryBinding, VisibilityHistorySnapshot};
use std::sync::Arc;

use super::{FrameHistoryValidationKey, ViewportFrameHistory};

impl ViewportFrameHistory {
    pub(crate) fn update(
        &mut self,
        generation: u64,
        bindings: Vec<FrameHistoryBinding>,
        visibility: VisibilityHistorySnapshot,
        static_index: VisibilityStaticIndex,
        validation_key: Arc<FrameHistoryValidationKey>,
    ) {
        self.generation = generation;
        self.bindings = bindings;
        self.visibility = visibility;
        self.static_index = static_index;
        self.validation_key = validation_key;
    }
}
