use super::*;

#[test]
fn fitted_thumbnail_preview_stays_inside_its_fractional_surface() {
    let surface = FrameRect {
        x: 10.2,
        y: 8.4,
        width: 80.0,
        height: 60.0,
    };
    let preview = fitted_thumbnail_preview_image_rect(&surface, 100, 100)
        .expect("a square source image should fit a visible thumbnail surface");

    assert!(preview.x >= surface.x);
    assert!(preview.y >= surface.y);
    assert!(preview.right() <= surface.right());
    assert!(preview.bottom() <= surface.bottom());
    assert_eq!(preview.width, 59.0);
    assert_eq!(preview.height, 59.0);
}

#[test]
fn fitted_thumbnail_preview_rejects_collapsed_or_unknown_source_dimensions() {
    let surface = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 60.0,
    };

    assert!(fitted_thumbnail_preview_image_rect(&surface, 0, 100).is_none());
    assert!(fitted_thumbnail_preview_image_rect(
        &FrameRect {
            width: 0.0,
            ..surface
        },
        100,
        100,
    )
    .is_none());
}
