use crate::{FrameHistoryBinding, VisibilityHistorySnapshot};

use super::viewport_frame_history::ViewportFrameHistory;

impl ViewportFrameHistory {
    pub(crate) fn update(
        &mut self,
        generation: u64,
        bindings: Vec<FrameHistoryBinding>,
        visibility: VisibilityHistorySnapshot,
    ) {
        self.generation = generation;
        self.bindings = bindings;
        self.visibility = visibility;
    }
}
