pub(crate) mod anti_alias;
mod attachment_ops;
mod core;
mod deferred;
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

pub use core::SceneRenderer;
pub(crate) use graph_execution::RenderGraphLightGridReport;
pub use graph_execution::{
    ParticleGpuTransparentDrawContext, RenderGraphExecutionResources, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext,
};

pub(crate) use core::{create_depth_texture, OFFSCREEN_FORMAT};
pub(crate) use deferred::{GBUFFER_ALBEDO_FORMAT, GBUFFER_MATERIAL_FORMAT};
pub(crate) use lighting::{
    light_buffer::pack_lighting_extract, light_grid_pass::build_light_grid_for_frame,
};
pub(in crate::graphics::scene) use mesh::skinning::SkinnedMeshJointPaletteUniform;
pub(crate) use mesh::{
    create_mesh_prewarm_validation_pipeline_layout,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    validate_mesh_prewarm_request_render_pipeline, MeshPipelineShaderSource, FALLBACK_MESH_SHADER,
};
#[cfg(test)]
pub(crate) use overlay::ViewportOverlayRenderer;
pub(crate) use post_process::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size,
    SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT,
    SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_FORMAT,
    SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION_FORMAT,
};
pub(crate) use prepass::NORMAL_FORMAT;
pub(crate) use shadow::atlas::{ShadowAtlasAllocator, ShadowAtlasResourceConfig};
pub(crate) use shadow::cascade::{
    cascade_shadow_bounds_from_camera_slice, compute_cascade_ranges, CascadeRange,
    CascadeSplitConfig,
};
pub(crate) use shadow::{build_shadow_frame_plan, ShadowLightSlotAssignment};
