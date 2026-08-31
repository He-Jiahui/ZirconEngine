//! Scene rasterization and resource streaming.

mod gpu_scene;
#[cfg(test)]
mod render_product_material_property_tests;
#[cfg(test)]
mod render_product_streamer_tests;
#[cfg(test)]
mod render_product_zshader_import_tests;
pub(crate) mod render_scene;
pub(in crate::graphics) mod resources;
#[path = "scene_renderer/mod.rs"]
mod scene_renderer;

pub(crate) use resources::{PipelineKey, ResourceStreamer, default_pipeline_key};
#[cfg(test)]
pub(crate) use scene_renderer::ViewportOverlayRenderer;
pub(in crate::graphics) use scene_renderer::environment::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_CUBE_PASS,
    IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_SH9_PASS, IBL_BAKE_PMREM_EXECUTOR_ID,
    IBL_BAKE_SOURCE_CUBEMAP_RESOURCE, append_ibl_bake_artifact_graph_plan,
    ibl_bake_pmrem_pass_name,
};
pub(in crate::graphics) use scene_renderer::transparency::{
    HALF_RES_TRANSPARENCY_COMPOSITE_EXECUTOR_ID, HALF_RES_TRANSPARENCY_COMPOSITE_PASS_NAME,
    HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_EXECUTOR_ID,
    HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_PASS_NAME, HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID,
    HALF_RES_TRANSPARENCY_MESH_PASS_NAME, HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID,
    half_resolution_transparency_supported,
};
pub(in crate::graphics) use scene_renderer::{
    AsyncViewportCaptureRequest, EnvironmentCapturePersistenceSubmission,
    EnvironmentCapturePersistenceSubmissionStatus, EnvironmentCaptureProbePublication,
    EnvironmentCaptureResidentOutput, EnvironmentCaptureSourceSubmission,
    EnvironmentCaptureSourceSubmissionStatus, EnvironmentCaptureSubmission,
    ViewportAsyncCaptureSubmission,
};
pub(crate) use scene_renderer::{
    CascadeRange, CascadeSplitConfig, FALLBACK_MESH_SHADER, FINAL_COLOR_FORMAT,
    GBUFFER_ALBEDO_FORMAT, GBUFFER_EMISSIVE_FORMAT, GBUFFER_MATERIAL_FORMAT,
    MeshPipelineShaderSource, NORMAL_FORMAT, RenderGraphLightGridReport, SCENE_COLOR_HDR_FORMAT,
    ShadowAtlasAllocator, ShadowAtlasResourceConfig, ShadowLightSlotAssignment, anti_alias,
    build_light_grid_for_frame, build_shadow_frame_plan, cascade_shadow_bounds_from_camera_slice,
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, compute_cascade_ranges,
    create_depth_texture, create_mesh_prewarm_validation_pipeline_layout, lighting,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    pack_lighting_extract, validate_mesh_prewarm_request_render_pipeline,
};
pub use scene_renderer::{
    IRRADIANCE_VOLUME_BIND_EXECUTOR_ID, IRRADIANCE_VOLUME_RESOURCE,
    LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID, LIGHT_COOKIE_ATLAS_RESOURCE, OIT_DRAW_SHADER_SOURCE,
    OIT_FRAGMENT_STORE_EXECUTOR_ID, OIT_RESOLVE_EXECUTOR_ID, OIT_RESOLVE_SHADER_SOURCE,
    PLANAR_FILTER_EXECUTOR_ID, PLANAR_REFLECTION_TEXTURE_RESOURCE, SSS_RECOMBINE_EXECUTOR_ID,
    SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID, VOLUMETRIC_INTEGRATE_EXECUTOR_ID,
    VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID, VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID,
    irradiance_volume_render_pass_executor_registrations,
    light_cookie_render_pass_executor_registrations, oit_render_pass_executor_registrations,
    planar_reflection_filter_compute_workload,
    planar_reflection_render_pass_executor_registrations, subsurface_render_feature_descriptor,
    subsurface_render_pass_executor_registrations, subsurface_scatter_compute_workload,
    subsurface_setup_compute_workload, volumetric_fog_render_pass_executor_registrations,
};
pub(in crate::graphics) use scene_renderer::{
    MeshHitProxyTokenSource, SSS_PARAMS_BUFFER_SIZE_BYTES, SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
    SceneHitProxyCompletion, SceneHitProxyProduct, SceneHitProxySubmission,
};
pub use scene_renderer::{
    ParticleGpuTransparentDrawContext, RenderGraphExecutionResources,
    RenderPassBufferUploadRecorder, RenderPassBufferUploadSink, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext, RenderPassGpuNativeContext, RenderPassRecordingPolicy,
};
pub use scene_renderer::{
    RealtimeIblCpuTimingReport, RealtimeIblFailureKind, RealtimeIblFailureOperation,
    RealtimeIblFailureReport, RealtimeIblGpuTimingReport, RealtimeIblReadiness,
    RealtimeIblStatusReport, RuntimeShaderPipelinePrewarmFailure,
    RuntimeShaderPipelinePrewarmReport, SceneRenderer, SceneRendererCoreStartupReport,
    SceneRendererDeferredLightingProfile, SceneRendererFrameTimingReport,
    SceneRendererGpuPassTiming, SceneRendererGpuTimingReport, SceneRendererStartupOptions,
    SceneRendererStartupReport,
};
