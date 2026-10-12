use bytemuck::{Pod, Zeroable};

use crate::core::math::Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
/// 场景局部清屏 shader 的颜色 ABI；由帧上传批次刷新后供本帧 clear pass 读取。
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
