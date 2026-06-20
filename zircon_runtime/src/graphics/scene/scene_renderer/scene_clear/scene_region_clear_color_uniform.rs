use bytemuck::{Pod, Zeroable};

use crate::core::math::Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct SceneRegionClearColorUniform {
    color: [f32; 4],
}

impl SceneRegionClearColorUniform {
    pub(super) fn new(color: Vec4) -> Self {
        Self {
            color: color.to_array(),
        }
    }
}
