mod create_depth_prepass_mesh_pipeline;
mod create_gbuffer_mesh_pipeline;
mod create_mesh_pipeline;
mod create_shadow_mesh_pipeline;
mod create_taa_reactive_mask_mesh_pipeline;
mod create_velocity_mesh_pipeline;
mod fallback_mesh_shader_source;
#[cfg(test)]
mod test_support;

pub(in crate::graphics::scene::scene_renderer::mesh) use create_depth_prepass_mesh_pipeline::create_depth_prepass_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_gbuffer_mesh_pipeline::create_gbuffer_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_mesh_pipeline::create_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_shadow_mesh_pipeline::create_shadow_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_taa_reactive_mask_mesh_pipeline::{
    create_taa_reactive_mask_mesh_pipeline, create_taa_reactive_material_mask_mesh_pipeline,
};
pub(in crate::graphics::scene::scene_renderer::mesh) use create_velocity_mesh_pipeline::create_velocity_mesh_pipeline;
pub(crate) use fallback_mesh_shader_source::FALLBACK_MESH_SHADER;
