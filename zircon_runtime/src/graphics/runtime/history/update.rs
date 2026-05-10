use crate::{FrameHistoryBinding, VisibilityHistorySnapshot};

use super::{FrameHistoryValidationKey, ViewportFrameHistory};

impl ViewportFrameHistory {
    pub(crate) fn update(
        &mut self,
        generation: u64,
        bindings: Vec<FrameHistoryBinding>,
        visibility: VisibilityHistorySnapshot,
        validation_key: FrameHistoryValidationKey,
    ) {
        self.generation = generation;
        self.bindings = bindings;
        self.visibility = visibility;
        self.validation_key = validation_key;
    }
}
