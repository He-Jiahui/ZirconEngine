use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use super::super::HostPaintImagePixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cached_visual_asset_pixels(
    key: &str,
) -> Option<Option<HostPaintImagePixels>> {
    let cache = visual_asset_cache();
    zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_lookup");
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(key)
        .cloned()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn store_visual_asset_pixels(
    key: String,
    pixels: Option<HostPaintImagePixels>,
) {
    let cache = visual_asset_cache();
    zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_store");
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, pixels);
}

fn visual_asset_cache() -> &'static Mutex<BTreeMap<String, Option<HostPaintImagePixels>>> {
    VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

static VISUAL_ASSET_CACHE: OnceLock<Mutex<BTreeMap<String, Option<HostPaintImagePixels>>>> =
    OnceLock::new();
