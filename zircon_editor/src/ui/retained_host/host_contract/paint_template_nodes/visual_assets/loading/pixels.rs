use std::path::PathBuf;

use super::super::super::sprite_atlas::resolve_editor_sprite_atlas_image;
use super::super::candidates::{first_existing_path, is_svg_path};
use super::super::svg::render_svg_file_pixels;
use super::super::{
    mui_icons, retained_image_pixels, HostPaintImagePixels, RasterTargetSize, MUI_ICON_DEFAULT_EDGE,
};
use super::cache::{cached_visual_asset_pixels, store_visual_asset_pixels};
use super::image::load_image_from_path;
use super::key::image_pixels_cache_key;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_pixels_from_candidates(
    candidates: impl FnOnce() -> Vec<PathBuf>,
    base_key: &str,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    // Asset refresh invalidates this bounded cache, so stable paints need neither candidate
    // construction nor filesystem probing.
    let key = image_pixels_cache_key(base_key, target, tint);
    if let Some(cached) = cached_visual_asset_pixels(&key) {
        zircon_runtime::profile_counter!("editor", "ui.visual_asset_cache.hit_count", 1);
        record_current_ui_perf_counter(UiPerfCounter::VisualAssetCacheHitCount, 1.0);
        return cached;
    }
    zircon_runtime::profile_counter!("editor", "ui.visual_asset_cache.miss_count", 1);
    zircon_runtime::profile_counter!("editor", "ui.visual_asset_cache.candidate_build_count", 1);
    record_current_ui_perf_counter(UiPerfCounter::VisualAssetCacheMissCount, 1.0);
    record_current_ui_perf_counter(UiPerfCounter::VisualAssetCacheCandidateBuildCount, 1.0);
    let candidates = candidates();
    let source_paths = candidates.clone();
    let path = {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_first_existing_path"
        );
        first_existing_path(candidates)
    };
    let Some(path) = path else {
        store_visual_asset_pixels(key, base_key, source_paths, None);
        return None;
    };

    let loaded = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_load_pixels");
        if mui_icons::is_module_path(&path) {
            let target = target.unwrap_or(RasterTargetSize {
                width: MUI_ICON_DEFAULT_EDGE,
                height: MUI_ICON_DEFAULT_EDGE,
            });
            mui_icons::render_module_pixels(&path, target, tint)
        } else if is_svg_path(&path) {
            target
                .and_then(|target| render_svg_file_pixels(&path, target, tint))
                .or_else(|| {
                    load_image_from_path(&path)
                        .and_then(|image| retained_image_pixels(&image, tint))
                })
        } else {
            load_image_from_path(&path).and_then(|image| retained_image_pixels(&image, tint))
        }
    }
    .map(|pixels| {
        if tint.is_none() {
            pixels.with_atlas(resolve_editor_sprite_atlas_image(base_key, &path))
        } else {
            pixels
        }
    })
    .map(
        |pixels| match icon_raster_resource_key(base_key, &pixels.resource_key) {
            Some(resource_key) => pixels.with_resource_key(resource_key),
            None => pixels,
        },
    );

    store_visual_asset_pixels(key, base_key, source_paths, loaded.clone());
    loaded
}

fn icon_raster_resource_key(base_key: &str, content_key: &str) -> Option<String> {
    (base_key.starts_with("icon:") || base_key.starts_with("template-icon:"))
        .then(|| format!("icon-raster:{content_key}"))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    use super::{icon_raster_resource_key, load_pixels_from_candidates};

    #[test]
    fn icon_raster_identity_can_preserve_the_content_addressed_pixel_key() {
        assert_eq!(
            icon_raster_resource_key("icon:save", "retained-image:16x16:abcd").as_deref(),
            Some("icon-raster:retained-image:16x16:abcd")
        );
        assert!(icon_raster_resource_key("image:preview", "retained-image:16x16:abcd").is_none());
    }

    #[test]
    fn warm_pixel_cache_skips_candidate_path_construction() {
        static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
        let base_key = format!(
            "test:warm-candidate-cache:{}",
            NEXT_KEY.fetch_add(1, Ordering::Relaxed)
        );
        let candidate_builds = AtomicUsize::new(0);

        assert!(load_pixels_from_candidates(
            || {
                candidate_builds.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            },
            &base_key,
            None,
            None,
        )
        .is_none());
        assert!(load_pixels_from_candidates(
            || {
                candidate_builds.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            },
            &base_key,
            None,
            None,
        )
        .is_none());

        assert_eq!(candidate_builds.load(Ordering::Relaxed), 1);
    }
}
