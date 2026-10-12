#[path = "budget/mod.rs"]
mod budget;
#[path = "capability_summary/mod.rs"]
mod capability_summary;
#[path = "capability_validation/mod.rs"]
mod capability_validation;
#[path = "capture_frame/mod.rs"]
mod capture_frame;
#[path = "profile/mod.rs"]
mod compile_options_for_profile;
#[path = "compiled_feature_names/mod.rs"]
mod compiled_feature_names;
#[path = "create_viewport/mod.rs"]
mod create_viewport;
#[path = "destroy_viewport/mod.rs"]
mod destroy_viewport;
#[path = "environment_capture_scheduler.rs"]
mod environment_capture_scheduler;
#[path = "environment_capture_submission.rs"]
mod environment_capture_submission;
#[path = "frame_profiler.rs"]
mod frame_profiler;
#[path = "capture/mod.rs"]
mod graphics_debugger_capture;
#[path = "pipelined/mod.rs"]
mod pipelined;
#[path = "../query_environment_runtime_snapshot/mod.rs"]
mod query_environment_runtime_snapshot;
#[path = "query_stats/mod.rs"]
mod query_stats;
#[path = "../query_virtual_geometry_debug_snapshot/mod.rs"]
mod query_virtual_geometry_debug_snapshot;
#[path = "../query_visible_spatial_snapshot/mod.rs"]
mod query_visible_spatial_snapshot;
#[path = "queue_capability/mod.rs"]
mod queue_capability;
#[path = "register_pipeline_asset/mod.rs"]
mod register_pipeline_asset;
#[path = "reload_pipeline/mod.rs"]
mod reload_pipeline;
#[path = "../render_framework_backend_error/mod.rs"]
mod render_framework_backend_error;
#[path = "state/mod.rs"]
mod render_framework_state;
#[path = "../render_framework_trait_binding/mod.rs"]
mod render_framework_trait_binding;
#[path = "set_pipeline_asset/mod.rs"]
mod set_pipeline_asset;
#[path = "set_quality_profile/mod.rs"]
mod set_quality_profile;
#[path = "extract/mod.rs"]
mod submit_frame_extract;
#[path = "submit_runtime_frame.rs"]
mod submit_runtime_frame;
#[path = "viewport_pick.rs"]
mod viewport_pick;
#[path = "viewport_record/mod.rs"]
mod viewport_record;
#[path = "viewport_surface/mod.rs"]
mod viewport_surface;
#[path = "wgpu_render_framework/mod.rs"]
mod wgpu_render_framework;
#[path = "../wgpu_render_framework_construct/mod.rs"]
mod wgpu_render_framework_construction;

pub(in crate::graphics) use environment_capture_scheduler::EnvironmentCaptureWorkItem;
#[cfg(test)]
pub(crate) use graphics_debugger_capture::renderdoc_capture_next_from_value;
pub use wgpu_render_framework::WgpuRenderFramework;
