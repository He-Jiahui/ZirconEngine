//! 后处理输出的读回验证诊断；与 graph/post_process 的节点数量诊断互补。
#[path = "color_lut.rs"]
mod color_lut;
#[path = "exposure.rs"]
mod exposure;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    exposure::record(store, frame_index, stats.last_exposure_readback_report);
    color_lut::record(store, frame_index, stats.last_color_lut_readback_report);
}

#[cfg(test)]
#[path = "../../post/tests/cases.rs"]
mod tests;
