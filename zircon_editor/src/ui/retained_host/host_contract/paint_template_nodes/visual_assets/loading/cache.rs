use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use super::super::HostPaintImagePixels;

const MAX_VISUAL_ASSET_CACHE_ENTRIES: usize = 256;
const MAX_VISUAL_ASSET_CACHE_BYTES: usize = 64 * 1024 * 1024;

struct VisualAssetCacheEntry {
    pixels: Option<HostPaintImagePixels>,
    byte_size: usize,
    last_used: u64,
}

#[derive(Default)]
struct VisualAssetCache {
    entries: BTreeMap<String, VisualAssetCacheEntry>,
    resident_bytes: usize,
    access_clock: u64,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cached_visual_asset_pixels(
    key: &str,
) -> Option<Option<HostPaintImagePixels>> {
    let cache = visual_asset_cache();
    zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_lookup");
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(key)
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn clear_visual_asset_pixels_cache(
) {
    VISUAL_ASSET_CACHE_GENERATION.fetch_add(1, Ordering::Relaxed);
    if let Some(cache) = VISUAL_ASSET_CACHE.get() {
        *cache.lock().unwrap_or_else(|poison| poison.into_inner()) = VisualAssetCache::default();
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn visual_asset_cache_generation(
) -> u64 {
    VISUAL_ASSET_CACHE_GENERATION.load(Ordering::Relaxed)
}

impl VisualAssetCache {
    fn get(&mut self, key: &str) -> Option<Option<HostPaintImagePixels>> {
        let last_used = self.next_access();
        self.entries.get_mut(key).map(|entry| {
            entry.last_used = last_used;
            entry.pixels.clone()
        })
    }

    fn insert(&mut self, key: String, pixels: Option<HostPaintImagePixels>) {
        self.remove(&key);
        let byte_size = visual_asset_byte_size(&pixels);
        if byte_size > MAX_VISUAL_ASSET_CACHE_BYTES {
            return;
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= MAX_VISUAL_ASSET_CACHE_ENTRIES
                || self.resident_bytes.saturating_add(byte_size) > MAX_VISUAL_ASSET_CACHE_BYTES)
        {
            let Some(evicted_key) = self
                .entries
                .iter()
                .min_by_key(|(key, entry)| (entry.last_used, *key))
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.remove(&evicted_key);
        }
        let last_used = self.next_access();
        self.resident_bytes = self.resident_bytes.saturating_add(byte_size);
        self.entries.insert(
            key,
            VisualAssetCacheEntry {
                pixels,
                byte_size,
                last_used,
            },
        );
    }

    fn remove(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            self.resident_bytes = self.resident_bytes.saturating_sub(entry.byte_size);
        }
    }

    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }
}

fn visual_asset_byte_size(pixels: &Option<HostPaintImagePixels>) -> usize {
    pixels.as_ref().map_or(0, |pixels| {
        pixels.rgba.len().saturating_add(
            pixels
                .atlas
                .as_ref()
                .and_then(|atlas| atlas.rgba.as_ref())
                .map_or(0, Vec::len),
        )
    })
}

fn visual_asset_cache() -> &'static Mutex<VisualAssetCache> {
    VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(VisualAssetCache::default()))
}

static VISUAL_ASSET_CACHE: OnceLock<Mutex<VisualAssetCache>> = OnceLock::new();
static VISUAL_ASSET_CACHE_GENERATION: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
mod tests {
    use super::{
        clear_visual_asset_pixels_cache, visual_asset_cache_generation, VisualAssetCache,
        MAX_VISUAL_ASSET_CACHE_ENTRIES,
    };

    #[test]
    fn cache_generation_advances_when_assets_are_refreshed() {
        let before = visual_asset_cache_generation();
        clear_visual_asset_pixels_cache();

        assert_ne!(visual_asset_cache_generation(), before);
    }

    #[test]
    fn cache_evicts_the_least_recently_used_entry_at_the_entry_budget() {
        let mut cache = VisualAssetCache::default();
        for index in 0..MAX_VISUAL_ASSET_CACHE_ENTRIES {
            cache.insert(format!("resource-{index:03}"), None);
        }
        assert!(cache.get("resource-000").is_some());

        cache.insert("resource-new".to_string(), None);

        assert_eq!(cache.entries.len(), MAX_VISUAL_ASSET_CACHE_ENTRIES);
        assert!(cache.get("resource-000").is_some());
        assert!(cache.get("resource-001").is_none());
        assert!(cache.get("resource-new").is_some());
    }
}
