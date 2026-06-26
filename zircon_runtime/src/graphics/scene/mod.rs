//! Scene rasterization and resource streaming.

mod gpu_scene;
#[cfg(test)]
mod render_product_material_property_tests;
#[cfg(test)]
mod render_product_streamer_tests;
#[cfg(test)]
mod render_product_zshader_import_tests;
mod resources;
#[path = "scene_renderer/mod.rs"]
mod scene_renderer;

#[cfg(test)]
pub(crate) use resources::ResourceStreamer;
pub(crate) use resources::{default_pipeline_key, PipelineKey};
pub use scene_renderer::SceneRenderer;
#[cfg(test)]
pub(crate) use scene_renderer::ViewportOverlayRenderer;
pub(crate) use scene_renderer::{
    anti_alias, build_light_grid_for_frame, build_shadow_frame_plan,
    cascade_shadow_bounds_from_camera_slice, cluster_buffer_bytes_for_size,
    cluster_dimensions_for_size, compute_cascade_ranges, create_depth_texture, lighting,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass, pack_lighting_extract,
    CascadeRange, CascadeSplitConfig, MeshPipelineShaderSource, RenderGraphLightGridReport,
    ShadowAtlasAllocator, ShadowAtlasResourceConfig, ShadowLightSlotAssignment,
    FALLBACK_MESH_SHADER, GBUFFER_ALBEDO_FORMAT, GBUFFER_MATERIAL_FORMAT, NORMAL_FORMAT,
    OFFSCREEN_FORMAT,
};
pub use scene_renderer::{
    ParticleGpuTransparentDrawContext, RenderGraphExecutionResources, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext,
};
