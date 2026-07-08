//! Scene rasterization and resource streaming.

mod gpu_scene;
#[cfg(test)]
mod render_product_material_property_tests;
#[cfg(test)]
mod render_product_streamer_tests;
#[cfg(test)]
mod render_product_zshader_import_tests;
pub(in crate::graphics) mod resources;
#[path = "scene_renderer/mod.rs"]
mod scene_renderer;

pub(crate) use resources::{default_pipeline_key, PipelineKey, ResourceStreamer};
pub(in crate::graphics) use scene_renderer::environment::ibl_bake_graph_plan::{
    append_ibl_bake_artifact_graph_plan, ibl_bake_pmrem_pass_name,
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_CUBE_PASS,
    IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_SH9_PASS, IBL_BAKE_PMREM_EXECUTOR_ID,
    IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
};
pub use scene_renderer::SceneRenderer;
#[cfg(test)]
pub(crate) use scene_renderer::ViewportOverlayRenderer;
pub(crate) use scene_renderer::{
    anti_alias, build_light_grid_for_frame, build_shadow_frame_plan,
    cascade_shadow_bounds_from_camera_slice, cluster_buffer_bytes_for_size,
    cluster_dimensions_for_size, compute_cascade_ranges, create_depth_texture,
    create_mesh_prewarm_validation_pipeline_layout, lighting,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    pack_lighting_extract, validate_mesh_prewarm_request_render_pipeline, CascadeRange,
    CascadeSplitConfig, MeshPipelineShaderSource, RenderGraphLightGridReport, ShadowAtlasAllocator,
    ShadowAtlasResourceConfig, ShadowLightSlotAssignment, FALLBACK_MESH_SHADER,
    GBUFFER_ALBEDO_FORMAT, GBUFFER_MATERIAL_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};
pub use scene_renderer::{
    ParticleGpuTransparentDrawContext, RenderGraphExecutionResources, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext,
};
