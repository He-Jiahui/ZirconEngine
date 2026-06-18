use super::candidates::icon_candidates;
use super::loading::load_image_from_candidates;
use super::*;

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

#[test]
fn template_svg_icon_pixels_follow_requested_target_size() {
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
    )
    .expect("template SVG icon should render at a requested large size");

    assert_eq!((small.width, small.height), (18, 18));
    assert_eq!((large.width, large.height), (54, 54));
    assert_ne!(small.rgba.len(), large.rgba.len());
    assert!(has_visible_pixel(&large));
}

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
    )
    .expect("missing template icons should produce deterministic fallback pixels");

    assert_eq!((missing.width, missing.height), (20, 20));
    assert!(missing
        .rgba
        .chunks_exact(4)
        .any(|pixel| pixel == ICON_TINT_ERROR.as_slice()));
}

#[test]
fn svg_font_scan_is_reserved_for_text_svg() {
    assert!(!svg::svg_may_need_fonts(
        br#"<svg viewBox="0 0 16 16"><path d="M0 0h16v16H0z"/></svg>"#
    ));
    assert!(svg::svg_may_need_fonts(
        br#"<svg viewBox="0 0 16 16"><text x="0" y="12">A</text></svg>"#
    ));
    assert!(svg::svg_may_need_fonts(
        br#"<svg viewBox="0 0 16 16"><path style="font-family:Arial" /></svg>"#
    ));
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
    )
    .expect("plain Image nodes should consume projected preview pixels");

    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(&image.rgba[0..4], &[201, 42, 33, 255]);
}

#[test]
fn template_icon_tint_uses_material_state_priority() {
    assert_eq!(
        template_image_tint(true, true, true, "error", "error", Some(ICON_TINT_ACTIVE)),
        Some(ICON_TINT_DISABLED)
    );
    assert_eq!(
        template_image_tint(true, true, false, "", "error", Some(ICON_TINT_ACTIVE)),
        Some(ICON_TINT_ERROR)
    );
    assert_eq!(
        template_image_tint(
            true,
            true,
            false,
            "warning",
            "normal",
            Some(ICON_TINT_ACTIVE),
        ),
        Some(ICON_TINT_WARNING)
    );
    assert_eq!(
        template_image_tint(true, true, false, "", "normal", Some(ICON_TINT_ERROR)),
        Some(ICON_TINT_ERROR)
    );
    assert_eq!(
        template_image_tint(true, true, false, "", "normal", None),
        Some(ICON_TINT_ACTIVE)
    );
    assert_eq!(
        template_image_tint(false, true, false, "error", "error", Some(ICON_TINT_ERROR)),
        None
    );
}

fn has_visible_pixel(image: &HostPaintImagePixels) -> bool {
    image.rgba.chunks_exact(4).any(|pixel| pixel[3] > 0)
}

const EDITOR_PAGES_WIRED_TEMPLATE_ICONS: &[&str] = &[
    "editor_pages/workbench/menu/open-project.svg",
    "editor_pages/workbench/menu/save-all.svg",
    "editor_pages/workbench/dock/reset-layout.svg",
    "editor_pages/asset_browser/navigation/folder.svg",
    "editor_pages/hierarchy/entity/scene.svg",
    "editor_pages/console_profiler/logs/log-info.svg",
    "editor_pages/scene_viewport/tools/universal-transform.svg",
    "editor_pages/scene_viewport/display/lit.svg",
    "editor_pages/scene_viewport/display/grid-overlay.svg",
    "editor_pages/scene_viewport/snapping/grid-snap.svg",
    "editor_pages/scene_viewport/snapping/angle-snap.svg",
    "editor_pages/scene_viewport/snapping/scale-snap.svg",
    "editor_pages/scene_viewport/display/gizmo-visibility.svg",
    "editor_pages/scene_viewport/camera/frame-selection.svg",
    "editor_pages/scene_viewport/play/play.svg",
    "editor_pages/scene_viewport/play/stop.svg",
    "editor_pages/scene_viewport/camera/perspective.svg",
    "editor_pages/asset_browser/import_pipeline/import-settings.svg",
    "editor_pages/asset_browser/references/reference.svg",
    "editor_pages/asset_browser/navigation/search.svg",
    "editor_pages/asset_browser/import_pipeline/import.svg",
    "editor_pages/asset_browser/navigation/recent.svg",
    "editor_pages/workbench/tabs/close-tab.svg",
    "editor_pages/graph_editor/nodes/state-node.svg",
    "editor_pages/animation_timeline/transport/timeline-play.svg",
    "editor_pages/console_profiler/profiling/frame-time.svg",
    "editor_pages/console_profiler/diagnostics/watch.svg",
    "editor_pages/build_plugins/package/package.svg",
    "editor_pages/build_plugins/plugins/plugin.svg",
];

struct IconReadabilityFootprint {
    visible_pixels: usize,
    span_width: u32,
    span_height: u32,
}

fn icon_readability_footprint(image: &HostPaintImagePixels) -> Option<IconReadabilityFootprint> {
    let mut visible_pixels = 0usize;
    let mut min_x = image.width;
    let mut min_y = image.height;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in 0..image.height {
        for x in 0..image.width {
            let alpha = image.rgba[((y * image.width + x) as usize * 4) + 3];
            if alpha == 0 {
                continue;
            }
            visible_pixels += 1;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    (visible_pixels > 0).then_some(IconReadabilityFootprint {
        visible_pixels,
        span_width: max_x - min_x + 1,
        span_height: max_y - min_y + 1,
    })
}

fn solid_preview_image(color: [u8; 4]) -> crate::ui::retained_host::primitives::Image {
    let pixels = [color, color, color, color].concat();
    crate::ui::retained_host::primitives::Image::from_rgba8(
        crate::ui::retained_host::primitives::SharedPixelBuffer::<
            crate::ui::retained_host::primitives::Rgba8Pixel,
        >::clone_from_slice(&pixels, 2, 2),
    )
}
