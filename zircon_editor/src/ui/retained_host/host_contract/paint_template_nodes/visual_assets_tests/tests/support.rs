use super::super::HostPaintImagePixels;

pub(super) fn has_visible_pixel(image: &HostPaintImagePixels) -> bool {
    image.rgba.chunks_exact(4).any(|pixel| pixel[3] > 0)
}

pub(super) const EDITOR_PAGES_WIRED_TEMPLATE_ICONS: &[&str] = &[
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

pub(super) struct IconReadabilityFootprint {
    pub(super) visible_pixels: usize,
    pub(super) span_width: u32,
    pub(super) span_height: u32,
}

pub(super) fn icon_readability_footprint(
    image: &HostPaintImagePixels,
) -> Option<IconReadabilityFootprint> {
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

pub(super) fn solid_preview_image(color: [u8; 4]) -> crate::ui::retained_host::primitives::Image {
    let pixels = [color, color, color, color].concat();
    crate::ui::retained_host::primitives::Image::from_rgba8(
        crate::ui::retained_host::primitives::SharedPixelBuffer::<
            crate::ui::retained_host::primitives::Rgba8Pixel,
        >::clone_from_slice(&pixels, 2, 2),
    )
}
