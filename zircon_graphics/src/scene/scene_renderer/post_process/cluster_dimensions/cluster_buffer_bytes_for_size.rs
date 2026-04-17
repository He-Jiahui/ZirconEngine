use zircon_math::UVec2;

use super::cluster_dimensions_for_size::cluster_dimensions_for_size;

pub(crate) fn cluster_buffer_bytes_for_size(size: UVec2) -> usize {
    let dimensions = cluster_dimensions_for_size(size);
    dimensions.x.max(1) as usize * dimensions.y.max(1) as usize * std::mem::size_of::<[f32; 4]>()
}
