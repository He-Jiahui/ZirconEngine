use crate::core::math::Vec4;

pub(super) fn raster_draws_for_mesh(
    mesh_index_count: u32,
    base_tint: Vec4,
) -> [(u32, u32, Vec4); 1] {
    [(0, mesh_index_count, base_tint)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_raster_draw_uses_fixed_storage() {
        let source = include_str!("raster_draws_for_mesh.rs");
        let product = source
            .split("#[cfg(test)]")
            .next()
            .expect("product source precedes tests");

        assert!(!product.contains("vec!["));
        assert_eq!(raster_draws_for_mesh(12, Vec4::ONE), [(0, 12, Vec4::ONE)]);
    }
}
