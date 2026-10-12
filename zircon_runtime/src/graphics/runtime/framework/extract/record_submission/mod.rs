#[path = "../../../extract/record_submission/record.rs"]
mod record;
#[path = "../../../extract/record_submission/record_capture.rs"]
mod record_capture;
#[path = "../../../extract/record_submission/record_history.rs"]
mod record_history;
#[path = "../../../extract/record_submission/record_present.rs"]
mod record_present;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record::{
    record_submission, update_hybrid_gi_runtime, update_virtual_geometry_runtime,
};
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record_history::record_history;
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record_present::record_present_submission;

#[cfg(test)]
#[path = "../../../extract/tests/record_submission_cases.rs"]
mod tests;
