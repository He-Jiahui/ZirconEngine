mod constants;
mod create_depth_texture;
mod runtime_features;
mod scene_renderer;
mod scene_renderer_asset_access;
mod scene_renderer_completion;
mod scene_renderer_construct;
mod scene_renderer_core;
mod scene_renderer_core_construct;
mod scene_renderer_core_render_compiled_scene;
mod scene_renderer_core_render_scene;
mod scene_renderer_core_write_scene_uniform;
mod scene_renderer_environment_capture;
mod scene_renderer_history;
mod scene_renderer_hit_proxy;
mod scene_renderer_pipeline_prewarm;
mod scene_renderer_realtime_ibl_diagnostics;
mod scene_renderer_render;
mod scene_renderer_render_capture;
mod scene_renderer_render_with_pipeline;
mod scene_renderer_runtime_outputs;
mod scene_renderer_submission_failure;
mod scene_renderer_submission_metrics;
mod scene_renderer_target;
mod scene_renderer_texture_residency;
mod scene_renderer_viewport_surface;
mod scene_submission_completion_journal;
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
