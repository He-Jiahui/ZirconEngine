use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;
use std::sync::Arc;

use crate::graphics::visibility::VisibilityStaticIndex;
use crate::graphics::{FrameHistoryBinding, VisibilityHistorySnapshot};

use super::FrameHistoryValidationKey;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportFrameHistory {
    pub(super) handle: FrameHistoryHandle,
    pub(super) target_size: UVec2,
    pub(super) render_size: UVec2,
    pub(super) pipeline: RenderPipelineHandle,
    pub(super) generation: u64,
    pub(super) bindings: Vec<FrameHistoryBinding>,
    pub(super) visibility: VisibilityHistorySnapshot,
    pub(super) static_index: VisibilityStaticIndex,
    pub(super) validation_key: Arc<FrameHistoryValidationKey>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn frame_history_shares_the_wide_validation_key() {
        let history = include_str!("viewport_frame_history.rs");
        let record = include_str!(
            "../render_framework/submit_frame_extract/record_submission/record_history.rs"
        );
        let arc_contract = concat!("Arc<", "FrameHistoryValidationKey>");
        let deep_clone = concat!("history_validation_key()", ".clone()");

        assert!(history.contains(arc_contract));
        assert!(record.contains("history_validation_key_shared()"));
        assert!(!record.contains(deep_clone));
    }
}
