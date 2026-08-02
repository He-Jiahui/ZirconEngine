use super::super::{ICON_TINT, template_image_pixels};
use super::support::{EDITOR_PAGES_WIRED_TEMPLATE_ICONS, icon_readability_footprint};

#[test]
fn editor_pages_template_icons_have_readable_16px_raster_footprints() {
    let preview = crate::ui::retained_host::primitives::Image::default();
    for icon in EDITOR_PAGES_WIRED_TEMPLATE_ICONS {
        let pixels = template_image_pixels(&preview, "", icon, 16, 16, Some(ICON_TINT), false)
            .unwrap_or_else(|| panic!("{icon} should render through the template icon path"));
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
