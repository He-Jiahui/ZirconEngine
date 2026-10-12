#[path = "constants.rs"]
mod constants;
#[path = "create_depth_texture/mod.rs"]
mod create_depth_texture;
#[path = "runtime_features/mod.rs"]
mod runtime_features;
#[path = "renderer/mod.rs"]
mod scene_renderer;
#[path = "asset_access.rs"]
mod scene_renderer_asset_access;
#[path = "complete.rs"]
mod scene_renderer_completion;
#[path = "scene_renderer_construct/mod.rs"]
mod scene_renderer_construct;
#[path = "core/mod.rs"]
mod scene_renderer_core;
#[path = "construct/mod.rs"]
mod scene_renderer_core_construct;
#[path = "render/mod.rs"]
mod scene_renderer_core_render_compiled_scene;
#[path = "render_scene/mod.rs"]
mod scene_renderer_core_render_scene;
#[path = "write_scene_uniform/mod.rs"]
mod scene_renderer_core_write_scene_uniform;
#[path = "environment_capture.rs"]
mod scene_renderer_environment_capture;
#[path = "scene_renderer_history/mod.rs"]
mod scene_renderer_history;
#[path = "scene_renderer_hit_proxy.rs"]
mod scene_renderer_hit_proxy;
#[path = "pipeline_prewarm.rs"]
mod scene_renderer_pipeline_prewarm;
#[path = "realtime_ibl_diagnostics.rs"]
mod scene_renderer_realtime_ibl_diagnostics;
#[path = "scene_renderer_render/mod.rs"]
mod scene_renderer_render;
#[path = "render_capture.rs"]
mod scene_renderer_render_capture;
#[path = "render_scene.rs"]
mod scene_renderer_render_scene;
#[path = "pipeline/mod.rs"]
mod scene_renderer_render_with_pipeline;
#[path = "outputs/mod.rs"]
mod scene_renderer_runtime_outputs;
#[path = "submission_failure.rs"]
mod scene_renderer_submission_failure;
#[path = "submission_metrics.rs"]
mod scene_renderer_submission_metrics;
#[path = "scene_renderer_target/mod.rs"]
mod scene_renderer_target;
#[path = "texture_residency.rs"]
mod scene_renderer_texture_residency;
#[path = "viewport_surface.rs"]
mod scene_renderer_viewport_surface;
#[path = "submission_complete_journal.rs"]
mod scene_submission_completion_journal;
#[path = "target_extent/mod.rs"]
mod target_extent;

pub use scene_renderer::{
    SceneRenderer, SceneRendererCoreStartupReport, SceneRendererDeferredLightingProfile,
    SceneRendererFrameTimingReport, SceneRendererGpuPassTiming, SceneRendererGpuTimingReport,
    SceneRendererStartupOptions, SceneRendererStartupReport,
};

pub(crate) use constants::{DEPTH_FORMAT, FINAL_COLOR_FORMAT, SCENE_COLOR_HDR_FORMAT};
pub(crate) use create_depth_texture::create_depth_texture;
pub(in crate::graphics::scene::scene_renderer) use scene_renderer_core_construct::material_texture_bind_group_layout_entries;
pub(crate) use scene_renderer_hit_proxy::{
    SceneHitProxyCompletion, SceneHitProxyProduct, SceneHitProxySubmission,
};
pub(in crate::graphics) use scene_renderer_render_with_pipeline::{
    AsyncViewportCaptureRequest, ViewportAsyncCaptureSubmission,
};
