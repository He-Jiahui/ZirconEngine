mod clear_render_target;
mod cluster_dimensions;
mod constants;
mod fallback_texture;
mod gpu_data;
mod params;
mod pass_graph;
mod resources;
mod scene_post_process_resources;
mod scene_runtime_feature_flags;

use crate::core::math::UVec2;

use gpu_data::{
    clustered_directional_light, hybrid_gi_probe_gpu, hybrid_gi_trace_region_gpu,
    reflection_probe_gpu,
};
use params::{
    bloom_params, cluster_params, depth_of_field_prepare_params, motion_vector_camera_params,
    post_process_params, ssao_params,
};

pub(crate) use cluster_dimensions::{cluster_buffer_bytes_for_size, cluster_dimensions_for_size};
pub(crate) use constants::{
    SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE_FORMAT,
    SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_FORMAT,
    SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT,
    SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_FORMAT,
    SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION_FORMAT,
};
pub(crate) use pass_graph::execute_post_process_pass_graph;
pub(in crate::graphics::scene::scene_renderer) use params::motion_vector_camera_params::MotionVectorCameraParams;
pub(crate) use scene_post_process_resources::ScenePostProcessResources;
pub(crate) use scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

pub(in crate::graphics::scene::scene_renderer) fn ssao_dispatch_groups(
    viewport_size: UVec2,
) -> [u32; 3] {
    [
        viewport_size
            .x
            .max(1)
            .div_ceil(constants::SSAO_WORKGROUP_SIZE),
        viewport_size
            .y
            .max(1)
            .div_ceil(constants::SSAO_WORKGROUP_SIZE),
        1,
    ]
}

pub(in crate::graphics::scene::scene_renderer) fn ssao_workgroup_size() -> [u32; 3] {
    [
        constants::SSAO_WORKGROUP_SIZE,
        constants::SSAO_WORKGROUP_SIZE,
        1,
    ]
}

pub(in crate::graphics::scene::scene_renderer) fn clustered_lighting_dispatch_groups(
    cluster_dimensions: UVec2,
) -> [u32; 3] {
    [
        cluster_dimensions
            .x
            .max(1)
            .div_ceil(constants::CLUSTER_WORKGROUP_SIZE),
        cluster_dimensions
            .y
            .max(1)
            .div_ceil(constants::CLUSTER_WORKGROUP_SIZE),
        1,
    ]
}

pub(in crate::graphics::scene::scene_renderer) fn clustered_lighting_workgroup_size() -> [u32; 3] {
    [
        constants::CLUSTER_WORKGROUP_SIZE,
        constants::CLUSTER_WORKGROUP_SIZE,
        1,
    ]
}
