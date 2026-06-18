use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::super::sprite_atlas::resolve_editor_sprite_atlas_image;
use super::candidates::{first_existing_path, is_svg_path};
use super::svg::{render_svg_file_image, render_svg_file_pixels};
use super::{
    mui_icons, retained_image_pixels, HostPaintImagePixels, RasterTargetSize, ICON_TINT,
    MUI_ICON_DEFAULT_EDGE,
};

pub(super) fn load_pixels_from_candidates(
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
    let cache = VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_lookup");
        if let Some(cached) = cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(&key)
        {
            return cached.clone();
        }
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

    {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_store");
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(key, loaded.clone());
    }
    loaded
}

pub(super) fn load_image_from_candidates(
    candidates: Vec<PathBuf>,
) -> Option<crate::ui::retained_host::primitives::Image> {
    for path in candidates {
        if let Some(image) = load_image_from_path(&path) {
            return Some(image);
        }
    }
    None
}

pub(super) fn missing_icon_pixels(
    base_key: &str,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let color = tint.unwrap_or(ICON_TINT);
    let mut rgba = vec![0; target.width as usize * target.height as usize * 4];
    let edge = target.width.min(target.height);
    let stroke = (edge / 10).clamp(1, 3);
    let max_x = target.width.saturating_sub(1);
    let max_y = target.height.saturating_sub(1);

    for y in 0..target.height {
        for x in 0..target.width {
            let border = x < stroke
                || y < stroke
                || max_x.saturating_sub(x) < stroke
                || max_y.saturating_sub(y) < stroke;
            let diagonal = x.abs_diff(y) < stroke || x.abs_diff(max_y.saturating_sub(y)) < stroke;
            if !border && !diagonal {
                continue;
            }
            let offset = ((y * target.width + x) as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    let image = HostPaintImagePixels {
        resource_key: format!("missing-icon:{base_key}:{}x{}", target.width, target.height),
        width: target.width,
        height: target.height,
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}

fn load_image_from_path(path: &Path) -> Option<crate::ui::retained_host::primitives::Image> {
    if !path.exists() {
        return None;
    }
    if mui_icons::is_module_path(path) {
        return mui_icons::render_module_image(path);
    }
    if is_svg_path(path) {
        return render_svg_file_image(path);
    }
    let image =
        crate::ui::retained_host::primitives::Image::load_from_path(path).unwrap_or_default();
    let size = image.size();
    (size.width > 0 && size.height > 0).then_some(image)
}

fn image_pixels_cache_key(
    base_key: &str,
    path: &Path,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> String {
    let size_key = target
        .map(|target| format!("{}x{}", target.width, target.height))
        .unwrap_or_else(|| "intrinsic".to_string());
    let tint_key = tint
        .map(|tint| {
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                tint[0], tint[1], tint[2], tint[3]
            )
        })
        .unwrap_or_else(|| "none".to_string());
    format!("{base_key}:{size_key}:tint:{tint_key}:{}", path.display())
}

static VISUAL_ASSET_CACHE: OnceLock<Mutex<BTreeMap<String, Option<HostPaintImagePixels>>>> =
    OnceLock::new();
