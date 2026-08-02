use std::collections::BTreeSet;

use super::support::{changed_pixels, paint_weight_heatmap};

#[test]
fn weight_heatmap_paints_a_continuous_multicolor_field_and_source_markers() {
    let bytes = paint_weight_heatmap(240, 150);
    let colors = bytes
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
        .collect::<BTreeSet<_>>();

    assert!(changed_pixels(&bytes, [0, 0, 0, 255]) > 18_000);
    assert!(colors.len() > 40, "expected a multicolor heat field");
    assert!(
        bytes
            .chunks_exact(4)
            .any(|pixel| pixel[0] > 180 && pixel[1] < 120)
    );
    assert!(
        bytes
            .chunks_exact(4)
            .any(|pixel| pixel[2] > 120 && pixel[0] < 80)
    );
}

#[test]
fn weight_heatmap_geometry_scales_with_its_available_frame() {
    let compact = paint_weight_heatmap(140, 90);
    let wide = paint_weight_heatmap(360, 210);

    assert!(changed_pixels(&compact, [0, 0, 0, 255]) > 5_000);
    assert!(changed_pixels(&wide, [0, 0, 0, 255]) > changed_pixels(&compact, [0, 0, 0, 255]));
}
