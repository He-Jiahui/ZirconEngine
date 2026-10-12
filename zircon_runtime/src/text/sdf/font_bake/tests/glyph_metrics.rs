use super::*;

#[test]
fn sdf_parser_metrics_preserve_recorded_layout_sizes_bearings_and_baselines() {
    // Frozen from fontsdf 0.5.3 with the repository's exact font fixtures.
    // Keep negative bearings, whitespace and accented outlines in the sample.
    let mono = include_bytes!("../../../../../assets/fonts/FiraMono-subset.ttf");
    let sans = include_bytes!("../../../../../assets/fonts/FiraSans-Regular.ttf");
    let cases = [
        (
            mono.as_slice(),
            13.0,
            'A',
            (17, 18, -5.0, -5.0, 7.8, 12.155001),
        ),
        (
            mono.as_slice(),
            32.0,
            'j',
            (31, 52, -8.0, -17.0, 19.2, 29.920002),
        ),
        (
            mono.as_slice(),
            64.0,
            'g',
            (68, 87, -15.0, -32.0, 38.4, 59.840004),
        ),
        (
            mono.as_slice(),
            32.0,
            ' ',
            (0, 0, -10.0, -10.0, 19.2, 29.920002),
        ),
        (
            mono.as_slice(),
            64.0,
            'O',
            (69, 81, -16.0, -19.0, 38.4, 59.840004),
        ),
        (
            sans.as_slice(),
            13.0,
            'j',
            (12, 22, -6.0, -8.0, 7.8, 12.155001),
        ),
        (
            sans.as_slice(),
            32.0,
            'A',
            (37, 42, -10.0, -10.0, 19.2, 29.920002),
        ),
        (
            sans.as_slice(),
            64.0,
            'j',
            (49, 99, -19.0, -32.0, 38.4, 59.840004),
        ),
        (
            sans.as_slice(),
            32.0,
            'é',
            (33, 46, -9.0, -11.0, 19.2, 29.920002),
        ),
        (
            sans.as_slice(),
            64.0,
            'O',
            (73, 81, -15.0, -19.0, 44.288002, 59.840004),
        ),
    ];
    for (bytes, px, ch, expected) in cases {
        let face = ttf_parser::Face::parse(bytes, 0).unwrap();
        let metrics = glyph_metrics(&face, px, face.glyph_index(ch).unwrap());
        assert_eq!(
            (
                metrics.bitmap_width,
                metrics.bitmap_height,
                metrics.bitmap_left,
                metrics.bitmap_bottom
            ),
            (expected.0, expected.1, expected.2, expected.3),
            "glyph {ch:?} at {px}px"
        );
        assert!((metrics.advance - expected.4).abs() < 0.0001);
        assert!((metrics.ascent - expected.5).abs() < 0.0001);
    }
}
