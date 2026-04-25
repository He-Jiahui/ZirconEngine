use crate::graphics::types::ViewportRenderFrame;

use super::runtime_parent_chain::{frame_has_runtime_scene_truth, scheduled_live_trace_region_ids};

pub(super) fn count_scheduled_trace_regions(frame: &ViewportRenderFrame) -> u32 {
    if frame.hybrid_gi_scene_prepare.is_some() || frame_has_runtime_scene_truth(frame) {
        return 0;
    }

    scheduled_live_trace_region_ids(frame).len() as u32
}
