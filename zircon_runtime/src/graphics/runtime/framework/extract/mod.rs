#[path = "submission/mod.rs"]
mod build_frame_submission_context;
#[path = "../../extract/frame_submission_context.rs"]
mod frame_submission_context;
#[path = "../../extract/prepare_runtime_submission/mod.rs"]
mod prepare_runtime_submission;
#[path = "../../extract/prepared_runtime_submission.rs"]
mod prepared_runtime_submission;
#[path = "record_submission/mod.rs"]
mod record_submission;
#[path = "runtime_feedback_batch.rs"]
mod runtime_feedback_batch;
#[path = "../../extract/submission_record_update.rs"]
mod submission_record_update;
#[path = "submit/mod.rs"]
mod submit;
#[path = "update_stats/mod.rs"]
mod update_stats;
#[path = "viewport_generate_guard.rs"]
mod viewport_generation_guard;

pub(in crate::graphics::runtime::render_framework) use submit::{
    present_frame_extract, present_frame_extract_with_ui, submit_frame_extract,
    submit_frame_extract_with_ui, submit_runtime_frame,
};
