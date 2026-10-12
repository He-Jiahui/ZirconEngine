use crate::core::math::{RenderVec3, RenderVec4, Vec3, Vec4};
use bytemuck::{Pod, Zeroable};

use super::super::fallback::{render_vec3_or, render_vec4_or};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
/// 选中框、网格、线框与 gizmo 共用的线条顶点；非有限输入在进入 GPU f32 表示时采用回退值。
pub(crate) struct LineVertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: [f32; 4],
}

impl LineVertex {
    pub(crate) fn new(position: Vec3, color: Vec4) -> Self {
        Self {
            position: render_vec3_or(position, RenderVec3::ZERO).to_array(),
            color: render_vec4_or(color, RenderVec4::ONE).to_array(),
        }
    }
}
