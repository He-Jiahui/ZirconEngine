mod color_lut;
mod exposure;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    exposure::record(store, frame_index, stats.last_exposure_readback_report);
    color_lut::record(store, frame_index, stats.last_color_lut_readback_report);
}

#[cfg(test)]
mod tests;
