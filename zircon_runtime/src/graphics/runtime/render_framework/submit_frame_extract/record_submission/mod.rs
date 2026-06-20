mod record;
mod record_capture;
mod record_history;
mod record_present;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record::{
    record_submission, update_hybrid_gi_runtime, update_virtual_geometry_runtime,
};
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record_history::record_history;
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record_present::record_present_submission;
