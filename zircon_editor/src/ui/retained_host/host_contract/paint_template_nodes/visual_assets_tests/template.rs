use super::super::candidates::icon_candidates;
use super::super::loading::load_image_from_candidates;
use super::super::{template_image_pixels, ICON_TINT, ICON_TINT_ERROR};
use super::support::{has_visible_pixel, solid_preview_image};

#[test]
fn template_svg_icon_pixels_share_resize_buckets() {
    let preview = load_image_from_candidates(icon_candidates("folder-open-outline"))
        .expect("test icon should load through the editor icon resolver");

    let small = template_image_pixels(
        &preview,
        "",
        "folder-open-outline",
        18,
        18,
        Some(ICON_TINT),
        false,
        None,
    )
    .expect("template SVG icon should render at a requested small size");
    let large = template_image_pixels(
        &preview,
        "",
        "folder-open-outline",
        54,
        54,
        Some(ICON_TINT),
        false,
        None,
    )
    .expect("template SVG icon should render at a requested large size");

    assert_eq!((small.width, small.height), (18, 18));
    assert_eq!((large.width, large.height), (56, 56));
    assert_ne!(small.rgba.len(), large.rgba.len());
    assert!(has_visible_pixel(&large));
}

#[test]
fn missing_bitmap_primary_uses_the_vector_fallback_bucket() {
    let preview = crate::ui::retained_host::primitives::Image::default();

    let fallback = template_image_pixels(
        &preview,
        "missing-primary.png",
        "folder-open-outline",
        54,
        54,
        Some(ICON_TINT),
        false,
        None,
    )
    .expect("a missing bitmap primary should resolve the semantic SVG icon fallback");

    assert_eq!((fallback.width, fallback.height), (56, 56));
}

#[test]
fn template_missing_icon_pixels_keep_visible_fallback() {
    let preview = crate::ui::retained_host::primitives::Image::default();

    let missing = template_image_pixels(
        &preview,
        "",
        "missing_zircon_mui_icon",
        20,
        20,
        Some(ICON_TINT_ERROR),
        false,
        None,
    )
    .expect("missing template icons should produce deterministic fallback pixels");

    assert_eq!((missing.width, missing.height), (20, 20));
    assert!(missing
        .rgba
        .chunks_exact(4)
        .any(|pixel| pixel == ICON_TINT_ERROR.as_slice()));
}

#[test]
fn template_plain_image_can_use_projected_preview_pixels_as_authority() {
    let preview = solid_preview_image([201, 42, 33, 255]);

    let image = template_image_pixels(
        &preview,
        "ui/editor/showcase_checker.svg",
        "",
        32,
        32,
        None,
        true,
        None,
    )
    .expect("plain Image nodes should consume projected preview pixels");

    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(&image.rgba[0..4], &[201, 42, 33, 255]);
}
