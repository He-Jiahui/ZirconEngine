use super::support::{changed_pixels, paint_sample_grid, pixel_at};

#[test]
fn sample_grid_paints_continuous_surface_grid_axes_and_points() {
    let bytes = paint_sample_grid(360, 260);

    assert!(changed_pixels(&bytes, [0, 0, 0, 255]) > 16_000);
    assert_ne!(
        pixel_at(&bytes, 360, 196, 56),
        pixel_at(&bytes, 360, 199, 56)
    );
}

#[test]
fn selected_sample_point_uses_cyan_accent_and_runtime_text_label() {
    let bytes = paint_sample_grid(360, 260);

    let cyan_pixels = bytes
        .chunks_exact(4)
        .filter(|pixel| pixel[1] > 145 && pixel[2] > 165 && pixel[0] < 80)
        .count();
    assert!(cyan_pixels > 24, "expected cyan selected-point pixels");

    let bright_label_pixels = bytes
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 160 && pixel[1] > 160 && pixel[2] > 160)
        .count();
    assert!(
        bright_label_pixels > 30,
        "expected Runtime Text label pixels"
    );
}

#[test]
fn sample_grid_geometry_scales_with_available_frame() {
    let compact = paint_sample_grid(220, 160);
    let wide = paint_sample_grid(520, 320);

    assert!(changed_pixels(&compact, [0, 0, 0, 255]) > 4_000);
    assert!(changed_pixels(&wide, [0, 0, 0, 255]) > changed_pixels(&compact, [0, 0, 0, 255]));
}
