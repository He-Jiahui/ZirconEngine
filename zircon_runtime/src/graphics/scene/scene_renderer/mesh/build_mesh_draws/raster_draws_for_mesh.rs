use crate::core::math::Vec4;

pub(super) fn raster_draws_for_mesh(
    mesh_index_count: u32,
    base_tint: Vec4,
) -> [(u32, u32, Vec4); 1] {
    [(0, mesh_index_count, base_tint)]
}

#[cfg(test)]
#[path = "tests/raster_draws_for_mesh.rs"]
mod tests;
