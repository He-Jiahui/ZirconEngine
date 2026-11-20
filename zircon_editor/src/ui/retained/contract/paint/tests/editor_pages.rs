use super::super::{template_image_pixels, ICON_TINT};
use super::support::{icon_readability_footprint, EDITOR_PAGES_WIRED_TEMPLATE_ICONS};

#[test]
fn editor_pages_template_icons_have_readable_16px_raster_footprints() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut bridge =
        crate::ui::retained_host::callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()
            .unwrap();
    let toolbar = bridge
        .paint_nodes_for_size(zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0))
        .unwrap();
    let mode = toolbar
        .iter()
        .find(|node| node.source_node_id.as_str() == "activate_scene_mode")
        .expect("authored tool mode node");
    let mode_icon = mode.icon_name.as_str();
    assert!(!mode_icon.is_empty(), "authored tool mode icon must be set");
    let preview = crate::ui::retained_host::primitives::Image::default();
    for icon in EDITOR_PAGES_WIRED_TEMPLATE_ICONS
        .iter()
        .copied()
        .chain(std::iter::once(mode_icon))
    {
        let pixels =
            template_image_pixels(&preview, "", icon, 16, 16, Some(ICON_TINT), false, None)
                .unwrap_or_else(|| panic!("{icon} should render through the template icon path"));
        if icon == mode_icon {
            assert!(
                !pixels.resource_key.starts_with("missing-icon:"),
                "{icon} must use the authored resource, not fallback pixels"
            );
        }
        let footprint = icon_readability_footprint(&pixels)
            .unwrap_or_else(|| panic!("{icon} should produce visible 16px pixels"));

        println!(
            "ICON_16PX_READABILITY icon={icon} visible={} span={}x{}",
            footprint.visible_pixels, footprint.span_width, footprint.span_height
        );
        assert_eq!((pixels.width, pixels.height), (16, 16));
        assert!(
            footprint.visible_pixels >= 12,
            "{icon} produced only {} visible pixels at 16px",
            footprint.visible_pixels
        );
        assert!(
            footprint.span_width >= 6 && footprint.span_height >= 6,
            "{icon} collapsed to a {}x{} footprint at 16px",
            footprint.span_width,
            footprint.span_height
        );
        assert!(
            footprint.visible_pixels < (pixels.width * pixels.height) as usize,
            "{icon} filled the whole 16px slot instead of a readable icon silhouette"
        );
    }
}
