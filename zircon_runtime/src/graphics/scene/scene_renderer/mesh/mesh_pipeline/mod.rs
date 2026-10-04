mod create_depth_prepass_mesh_pipeline;
mod create_gbuffer_mesh_pipeline;
mod create_hit_proxy_mesh_pipeline;
mod create_mesh_pipeline;
mod create_oit_mesh_pipeline;
mod create_shadow_mesh_pipeline;
mod create_taa_reactive_mask_mesh_pipeline;
mod create_velocity_mesh_pipeline;
mod fallback_mesh_shader_source;
#[cfg(test)]
#[path = "tests/test_support.rs"]
mod test_support;

pub(in crate::graphics::scene::scene_renderer::mesh) use create_depth_prepass_mesh_pipeline::create_depth_prepass_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_gbuffer_mesh_pipeline::create_gbuffer_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_hit_proxy_mesh_pipeline::create_hit_proxy_mesh_pipeline;
pub(crate) use create_hit_proxy_mesh_pipeline::{
    HIT_PROXY_TOKEN_FORMAT, HIT_PROXY_WORLD_NORMAL_FORMAT, HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT,
};
pub(in crate::graphics::scene::scene_renderer::mesh) use create_mesh_pipeline::create_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_oit_mesh_pipeline::create_oit_mesh_pipeline;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_shadow_mesh_pipeline::create_shadow_mesh_pipeline;
pub(crate) use create_taa_reactive_mask_mesh_pipeline::MESH_TAA_REACTIVE_MASK_TARGET_FORMAT;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_taa_reactive_mask_mesh_pipeline::{
    create_taa_reactive_mask_mesh_pipeline, create_taa_reactive_material_mask_mesh_pipeline,
};
pub(in crate::graphics::scene::scene_renderer::mesh) use create_velocity_mesh_pipeline::create_velocity_mesh_pipeline;
pub(crate) use create_velocity_mesh_pipeline::MESH_VELOCITY_TARGET_FORMAT;
pub(crate) use fallback_mesh_shader_source::FALLBACK_MESH_SHADER;

fn mesh_front_face(key: &crate::graphics::scene::resources::PipelineKey) -> wgpu::FrontFace {
    if key.reverse_raster_winding {
        wgpu::FrontFace::Cw
    } else {
        wgpu::FrontFace::Ccw
    }
}

#[cfg(test)]
#[path = "tests/mod_raster_state_tests.rs"]
mod raster_state_tests;
