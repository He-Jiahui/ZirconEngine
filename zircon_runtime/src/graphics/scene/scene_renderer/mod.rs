pub(crate) mod advanced_lighting;
pub(crate) mod anti_alias;
mod attachment_ops;
mod core;
mod deferred;
pub(in crate::graphics) mod environment;
mod graph_execution;
mod history;
mod hzb;
pub(crate) mod lighting;
mod mesh;
mod overlay;
mod particle;
mod post_process;
mod prepass;
mod primitives;
mod scene_clear;
mod shadow;
mod sprite;
mod temporal;
mod transparent;
mod ui;

pub use advanced_lighting::{
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
pub use core::{
    SceneRenderer, SceneRendererCoreStartupReport, SceneRendererDeferredLightingProfile,
    SceneRendererFrameTimingReport, SceneRendererStartupOptions, SceneRendererStartupReport,
    SceneViewportSurface,
};
pub use environment::RealtimeIblGpuTimingReport;
pub(crate) use graph_execution::RenderGraphLightGridReport;
pub use graph_execution::{
    ParticleGpuTransparentDrawContext, RenderGraphExecutionResources, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext,
};

pub(crate) use core::{FINAL_COLOR_FORMAT, SCENE_COLOR_HDR_FORMAT, create_depth_texture};
pub(crate) use deferred::{
    GBUFFER_ALBEDO_FORMAT, GBUFFER_EMISSIVE_FORMAT, GBUFFER_MATERIAL_FORMAT,
};
pub(crate) use lighting::{
    light_buffer::pack_lighting_extract, light_grid_pass::build_light_grid_for_frame,
};
pub(in crate::graphics::scene) use mesh::skinning::SkinnedMeshJointPaletteStorage;
pub(crate) use mesh::{
    FALLBACK_MESH_SHADER, MeshPipelineShaderSource, create_mesh_prewarm_validation_pipeline_layout,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    validate_mesh_prewarm_request_render_pipeline,
};
#[cfg(test)]
pub(crate) use overlay::ViewportOverlayRenderer;
pub(crate) use post_process::{
    SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT,
    SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_FORMAT,
    SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION_FORMAT, cluster_buffer_bytes_for_size,
    cluster_dimensions_for_size,
};
pub(crate) use prepass::NORMAL_FORMAT;
pub(crate) use shadow::atlas::{ShadowAtlasAllocator, ShadowAtlasResourceConfig};
pub(crate) use shadow::cascade::{
    CascadeRange, CascadeSplitConfig, cascade_shadow_bounds_from_camera_slice,
    compute_cascade_ranges,
};
pub(crate) use shadow::{ShadowLightSlotAssignment, build_shadow_frame_plan};
