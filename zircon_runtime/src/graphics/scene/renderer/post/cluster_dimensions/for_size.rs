use crate::core::math::UVec2;

use super::super::constants::CLUSTER_TILE_SIZE;

/// 按逻辑视口划分二维灯光聚合网格；离屏目标分配和后处理 dispatch 必须使用同一尺寸。
/// 每个单元覆盖 `CLUSTER_TILE_SIZE` 像素，零尺寸仍保留一个可绑定的单元。
pub(crate) fn cluster_dimensions_for_size(size: UVec2) -> UVec2 {
    UVec2::new(
        size.x.max(1).div_ceil(CLUSTER_TILE_SIZE),
        size.y.max(1).div_ceil(CLUSTER_TILE_SIZE),
    )
}
