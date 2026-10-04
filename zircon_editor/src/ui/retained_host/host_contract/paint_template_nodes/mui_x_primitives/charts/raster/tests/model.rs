use super::ChartRaster;

#[test]
fn half_coverage_white_over_black_resolves_in_linear_light() {
    let mut raster = ChartRaster {
        width: 1,
        height: 1,
        rgba: vec![0, 0, 0, 255],
    };

    raster.sample_pixel(0, 0, |x, _| (x < 0.5).then_some([255, 255, 255, 255]));

    assert!((187..=189).contains(&raster.rgba[0]));
    assert_eq!(raster.rgba[0], raster.rgba[1]);
    assert_eq!(raster.rgba[1], raster.rgba[2]);
    assert_eq!(raster.rgba[3], 255);
}
