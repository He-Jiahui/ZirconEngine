use super::ChartRaster;

#[test]
fn disc_edges_resolve_to_fractional_alpha_without_losing_the_opaque_center() {
    let mut raster = ChartRaster::transparent(8, 8);

    raster.draw_disc((4.0, 4.0), 3.5, [80, 160, 240, 255]);

    let alpha = raster
        .rgba
        .chunks_exact(4)
        .map(|pixel| pixel[3])
        .collect::<Vec<_>>();
    assert!(alpha.contains(&0));
    assert!(alpha.contains(&255));
    assert!(alpha.iter().any(|value| *value > 0 && *value < 255));
}
