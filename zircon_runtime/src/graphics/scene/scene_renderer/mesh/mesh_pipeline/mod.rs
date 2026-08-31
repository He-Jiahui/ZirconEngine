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
pub(crate) use create_velocity_mesh_pipeline::MESH_VELOCITY_TARGET_FORMAT;
pub(in crate::graphics::scene::scene_renderer::mesh) use create_velocity_mesh_pipeline::create_velocity_mesh_pipeline;
pub(crate) use fallback_mesh_shader_source::FALLBACK_MESH_SHADER;

fn mesh_front_face(key: &crate::graphics::scene::resources::PipelineKey) -> wgpu::FrontFace {
    if key.reverse_raster_winding {
        wgpu::FrontFace::Cw
    } else {
        wgpu::FrontFace::Ccw
    }
}

#[cfg(test)]
mod raster_state_tests {
    use crate::graphics::scene::resources::default_pipeline_key;

    #[test]
    fn mirrored_pipeline_key_reverses_the_raster_front_face() {
        let mut key = default_pipeline_key();
        assert_eq!(super::mesh_front_face(&key), wgpu::FrontFace::Ccw);

        key.reverse_raster_winding = true;
        assert_eq!(super::mesh_front_face(&key), wgpu::FrontFace::Cw);
    }

    #[test]
    fn every_mesh_raster_pass_consumes_the_shared_front_face_policy() {
        let pipeline_sources = [
            include_str!("create_depth_prepass_mesh_pipeline.rs"),
            include_str!("create_gbuffer_mesh_pipeline.rs"),
            include_str!("create_hit_proxy_mesh_pipeline.rs"),
            include_str!("create_mesh_pipeline.rs"),
            include_str!("create_oit_mesh_pipeline.rs"),
            include_str!("create_shadow_mesh_pipeline.rs"),
            include_str!("create_taa_reactive_mask_mesh_pipeline.rs"),
            include_str!("create_velocity_mesh_pipeline.rs"),
        ];

        for source in pipeline_sources {
            assert!(source.contains("front_face: super::mesh_front_face(key)"));
        }
    }
}
