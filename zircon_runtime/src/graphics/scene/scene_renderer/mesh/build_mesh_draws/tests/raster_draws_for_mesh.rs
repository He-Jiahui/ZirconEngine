use super::*;

#[test]
fn single_raster_draw_uses_fixed_storage() {
    let source = include_str!("../raster_draws_for_mesh.rs");
    let product = source
        .split("#[cfg(test)]")
        .next()
        .expect("product source precedes tests");

    assert!(!product.contains("vec!["));
    assert_eq!(raster_draws_for_mesh(12, Vec4::ONE), [(0, 12, Vec4::ONE)]);
}
