use super::super::load_visual_asset_pixels_for_size;
use super::support::has_visible_pixel;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

#[test]
fn runtime_svg_icon_pixels_follow_requested_target_size() {
    let icon = UiVisualAssetRef::Icon("folder-open-outline".to_string());

    let small = load_visual_asset_pixels_for_size(&icon, 16, 16)
        .expect("runtime SVG icon should render at a requested small size");
    let large = load_visual_asset_pixels_for_size(&icon, 48, 48)
        .expect("runtime SVG icon should render at a requested large size");

    assert_eq!((small.width, small.height), (16, 16));
    assert_eq!((large.width, large.height), (48, 48));
    assert_ne!(small.rgba.len(), large.rgba.len());
    assert!(has_visible_pixel(&large));
}
