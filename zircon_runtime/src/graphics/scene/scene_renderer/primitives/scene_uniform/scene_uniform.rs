use bytemuck::{Pod, Zeroable};

pub(crate) use crate::core::framework::render::BASIC_SCENE_UNIFORM_POINT_LIGHT_LIMIT as SCENE_UNIFORM_POINT_LIGHT_LIMIT;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub(crate) struct SceneUniform {
    pub(crate) view_proj: [[f32; 4]; 4],
    pub(crate) inverse_view_proj: [[f32; 4]; 4],
    pub(crate) light_dir: [f32; 4],
    pub(crate) light_color: [f32; 4],
    pub(crate) ambient_color: [f32; 4],
    pub(crate) previous_view_proj: [[f32; 4]; 4],
    pub(crate) motion_params: [f32; 4],
    pub(crate) point_light_position_range: [[f32; 4]; SCENE_UNIFORM_POINT_LIGHT_LIMIT],
    pub(crate) point_light_color_intensity: [[f32; 4]; SCENE_UNIFORM_POINT_LIGHT_LIMIT],
    pub(crate) point_light_params: [f32; 4],
}
