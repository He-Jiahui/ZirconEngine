use crate::core::math::UVec2;

use super::super::constants::CLUSTER_TILE_SIZE;

pub(crate) fn cluster_dimensions_for_size(size: UVec2) -> UVec2 {
    UVec2::new(
        size.x.max(1).div_ceil(CLUSTER_TILE_SIZE),
        size.y.max(1).div_ceil(CLUSTER_TILE_SIZE),
    )
}
