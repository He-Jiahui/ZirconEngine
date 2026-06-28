mod construct;
mod ensure_depth_prepass_pipeline;
mod ensure_gbuffer_pipeline;
mod ensure_pipeline;
mod ensure_shadow_pipeline;
mod ensure_taa_reactive_mask_pipeline;
mod ensure_velocity_pipeline;
mod forward_shadow_receiver;
mod mesh_pipeline_cache;
mod mesh_pipeline_variant_registry;
mod shader_source;

pub(crate) use mesh_pipeline_cache::MeshPipelineCache;
pub(crate) use mesh_pipeline_variant_registry::{
    MeshPipelineVariantRegistry, MeshPipelineVariantResolver,
};
#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer::mesh) use shader_source::{
    mesh_pipeline_deferred_gbuffer_template_source_for_geometry,
    mesh_pipeline_depth_prepass_template_source_for_geometry,
    mesh_pipeline_shadow_template_source_for_geometry,
    mesh_pipeline_taa_reactive_mask_template_source_for_geometry,
    mesh_pipeline_velocity_template_source_for_geometry,
};
pub(crate) use shader_source::{
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass, MeshPipelineShaderSource,
};
