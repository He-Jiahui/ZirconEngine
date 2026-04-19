use bytemuck::{Pod, Zeroable};
use crate::core::math::{RenderVec3, RenderVec4, Vec3, Vec4};

use super::super::fallback::{render_vec3_or, render_vec4_or};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct IconVertex {
    pub(crate) position: [f32; 3],
    pub(crate) uv: [f32; 2],
    pub(crate) color: [f32; 4],
}

impl IconVertex {
    pub(crate) fn new(position: Vec3, uv: [f32; 2], color: Vec4) -> Self {
        Self {
            position: render_vec3_or(position, RenderVec3::ZERO).to_array(),
            uv,
            color: render_vec4_or(color, RenderVec4::ONE).to_array(),
        }
    }
}
