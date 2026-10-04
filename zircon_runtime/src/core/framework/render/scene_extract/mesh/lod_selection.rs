use crate::core::math::Real;

/// 场景提取按相机距离选出的 LOD 结果；渲染队列应沿用该选择以保持同帧一致。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMeshLodSelection {
    pub level_index: u32,
    pub min_distance: Real,
}

impl RenderMeshLodSelection {
    pub fn new(level_index: u32, min_distance: Real) -> Self {
        Self {
            level_index,
            min_distance,
        }
    }
}
