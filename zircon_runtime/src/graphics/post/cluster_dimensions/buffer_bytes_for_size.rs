use crate::core::math::UVec2;

use super::cluster_dimensions_for_size::cluster_dimensions_for_size;

/// 为后处理灯光聚合结果预留空间，每个二维单元保存一个 `vec4<f32>`。
/// 该容量与 `cluster_dimensions_for_size` 配套，不能当作逐光源索引表的容量。
pub(crate) fn cluster_buffer_bytes_for_size(size: UVec2) -> usize {
    let dimensions = cluster_dimensions_for_size(size);
    dimensions.x.max(1) as usize * dimensions.y.max(1) as usize * std::mem::size_of::<[f32; 4]>()
}
