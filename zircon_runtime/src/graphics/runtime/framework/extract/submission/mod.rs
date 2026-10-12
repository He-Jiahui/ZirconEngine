#[path = "build.rs"]
mod build;
#[path = "../../../extract/submission/camera_history_key.rs"]
mod camera_history_key;
#[path = "../../../extract/submission/compile_pipeline.rs"]
mod compile_pipeline;
#[path = "../../../../submission/environment_ibl_compile_options.rs"]
mod environment_ibl_compile_options;
#[path = "../../../../extract/submission/material_feature_extract.rs"]
mod material_feature_extract;
#[path = "../../../../extract/submission/resolve_enabled_features.rs"]
mod resolve_enabled_features;
#[path = "../../../../extract/submission/resolve_viewport_record_state.rs"]
mod resolve_viewport_record_state;
#[path = "../../../../extract/submission/subsurface_profile_extract.rs"]
mod subsurface_profile_extract;
#[path = "../../../extract/submission/target_resolution.rs"]
mod target_resolution;
#[path = "../../../extract/submission/viewport_record_state.rs"]
mod viewport_record_state;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use build::build_frame_submission_context_from_runtime_frame_extract;
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use target_resolution::validate_camera_surface_present_target;
