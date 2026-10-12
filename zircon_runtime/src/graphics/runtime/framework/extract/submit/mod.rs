#[path = "../../../extract/submit/build_runtime_frame.rs"]
mod build_runtime_frame;
#[path = "debug.rs"]
mod build_virtual_geometry_debug_snapshot;
#[path = "camera_loop.rs"]
mod camera_loop;
#[path = "../../../extract/submit/collect_runtime_feedback.rs"]
mod collect_runtime_feedback;
#[path = "../../../extract/submit/completion_error_stats.rs"]
mod completion_error_stats;
#[path = "../../../extract/submit/present_frame_extract.rs"]
mod present_frame_extract;
#[path = "../../../extract/submit/publish_viewport_product.rs"]
mod publish_viewport_product;
#[path = "../../../extract/submit/record_camera_history.rs"]
mod record_camera_history;
#[path = "../../../extract/submit/release_previous_history.rs"]
mod release_previous_history;
#[path = "../../../extract/submit/resolve_history_handle.rs"]
mod resolve_history_handle;
#[path = "submit.rs"]
mod submit;
#[path = "../../../extract/submit/submit_runtime_frame.rs"]
mod submit_runtime_frame;
#[path = "../../../../extract/submit/update_particle_previous_state.rs"]
mod update_particle_previous_state;
#[path = "../../../../extract/submit/update_temporal_camera_history.rs"]
mod update_temporal_camera_history;

pub(in crate::graphics::runtime::render_framework) use present_frame_extract::{
    present_frame_extract, present_frame_extract_with_ui,
};
pub(in crate::graphics::runtime::render_framework) use submit::submit_frame_extract;
pub(in crate::graphics::runtime::render_framework) use submit::submit_frame_extract_with_ui;
pub(in crate::graphics::runtime::render_framework) use submit_runtime_frame::submit_runtime_frame;

#[cfg(test)]
#[path = "../../../extract/submit/tests/submit_extract_cases.rs"]
mod tests;
