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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_pixels_from_candidates(
    candidates: Vec<PathBuf>,
    base_key: &str,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let path = {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_first_existing_path"
        );
        first_existing_path(candidates)?
    };
    let key = image_pixels_cache_key(
        base_key,
        &path,
        target.filter(|_| is_svg_path(&path) || mui_icons::is_module_path(&path)),
        tint,
    );
    if let Some(cached) = cached_visual_asset_pixels(&key) {
        return cached;
    }

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
            pixels.with_resource_key(key.clone())
        }
    });

    store_visual_asset_pixels(key, loaded.clone());
    loaded
}
