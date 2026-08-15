use super::super::{load_visual_asset_pixels_for_size, template_image_pixels, ICON_TINT};
use super::support::has_visible_pixel;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

#[test]
fn mui_material_icon_modules_render_from_local_dev_source() {
    let add = UiVisualAssetRef::Icon("mui:Add".to_string());
    let add_pixels = load_visual_asset_pixels_for_size(&add, 24, 24)
        .expect("MUI Add icon should render from the local dev source");
    let add_large_pixels = load_visual_asset_pixels_for_size(&add, 36, 36)
        .expect("MUI Add icon should re-render when target size changes");
    assert_eq!((add_pixels.width, add_pixels.height), (24, 24));
    assert_eq!((add_large_pixels.width, add_large_pixels.height), (36, 36));
    assert!(has_visible_pixel(&add_pixels));

    let search = UiVisualAssetRef::Icon("@mui/icons-material/Search".to_string());
    let search_pixels = load_visual_asset_pixels_for_size(&search, 32, 32)
        .expect("prefixed MUI Search icon should render from the local dev source");
    assert_eq!((search_pixels.width, search_pixels.height), (32, 32));
    assert!(has_visible_pixel(&search_pixels));
}

#[test]
fn template_mui_icon_ligatures_render_from_local_dev_source() {
    let preview = crate::ui::retained_host::primitives::Image::default();

    let folder = template_image_pixels(&preview, "", "folder", 24, 24, Some(ICON_TINT), false)
        .expect("MUI Icon ligature should resolve to the local Material icon module");
    let add_circle =
        template_image_pixels(&preview, "", "add_circle", 32, 32, Some(ICON_TINT), false)
            .expect("snake-case MUI Icon ligature should resolve to PascalCase module source");

    assert_eq!((folder.width, folder.height), (24, 24));
    assert_eq!((add_circle.width, add_circle.height), (32, 32));
    assert!(has_visible_pixel(&folder));
    assert!(has_visible_pixel(&add_circle));
}
