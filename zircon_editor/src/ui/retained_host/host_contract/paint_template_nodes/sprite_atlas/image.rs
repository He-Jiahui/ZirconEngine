use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

const MAX_ATLAS_RGBA_CACHE_ENTRIES: usize = 64;
const MAX_ATLAS_RGBA_CACHE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone)]
struct AtlasRgba {
    resource_key: String,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct AtlasRgbaMetadata {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) generation: u64,
}

#[derive(Default)]
struct AtlasRgbaCache {
    entries: BTreeMap<PathBuf, AtlasRgba>,
    resource_index: BTreeMap<String, AtlasRgbaResourceIndex>,
    resident_bytes: usize,
}

struct AtlasRgbaResourceIndex {
    generation: u64,
    texture_path: PathBuf,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_atlas_rgba(
    manifest_path: &Path,
    atlas_texture: &str,
    width: u32,
    height: u32,
) -> Option<AtlasRgbaMetadata> {
    let texture_path = atlas_texture_path(manifest_path, atlas_texture)?;
    let cache = ATLAS_RGBA_CACHE.get_or_init(|| Mutex::new(AtlasRgbaCache::default()));
    let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(cached) = cache.entries.get(&texture_path) {
        return Some(AtlasRgbaMetadata {
            generation: cached.generation,
        });
    }
    let image = image::open(&texture_path).ok()?.into_rgba8();
    if image.dimensions() != (width, height) {
        return None;
    }
    let decoded = AtlasRgba {
        resource_key: atlas_texture.to_string(),
        rgba: image.into_raw(),
        generation: NEXT_ATLAS_RGBA_GENERATION.fetch_add(1, Ordering::Relaxed),
    };
    if decoded.rgba.len() > MAX_ATLAS_RGBA_CACHE_BYTES {
        return None;
    }
    while !cache.entries.is_empty()
        && (cache.entries.len() >= MAX_ATLAS_RGBA_CACHE_ENTRIES
            || cache.resident_bytes.saturating_add(decoded.rgba.len()) > MAX_ATLAS_RGBA_CACHE_BYTES)
    {
        let Some(evicted_key) = cache.entries.keys().next().cloned() else {
            break;
        };
        cache.remove(&evicted_key);
    }
    cache.resident_bytes = cache.resident_bytes.saturating_add(decoded.rgba.len());
    let metadata = AtlasRgbaMetadata {
        generation: decoded.generation,
    };
    cache.resource_index.insert(
        decoded.resource_key.clone(),
        AtlasRgbaResourceIndex {
            generation: decoded.generation,
            texture_path: texture_path.clone(),
        },
    );
    cache.entries.insert(texture_path, decoded);
    Some(metadata)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn copy_atlas_rgba(
    resource_key: &str,
    generation: u64,
) -> Option<Vec<u8>> {
    let cache = ATLAS_RGBA_CACHE.get()?;
    let cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    let indexed = cache.resource_index.get(resource_key)?;
    (indexed.generation == generation)
        .then(|| cache.entries.get(&indexed.texture_path))
        .flatten()
        .map(|entry| entry.rgba.clone())
}

impl AtlasRgbaCache {
    fn remove(&mut self, texture_path: &Path) {
        let Some(entry) = self.entries.remove(texture_path) else {
            return;
        };
        self.resident_bytes = self.resident_bytes.saturating_sub(entry.rgba.len());
        if self
            .resource_index
            .get(&entry.resource_key)
            .is_some_and(|indexed| {
                indexed.generation == entry.generation && indexed.texture_path == texture_path
            })
        {
            self.resource_index.remove(&entry.resource_key);
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn clear_atlas_rgba_cache() {
    if let Some(cache) = ATLAS_RGBA_CACHE.get() {
        *cache.lock().unwrap_or_else(|poison| poison.into_inner()) = AtlasRgbaCache::default();
    }
}

fn atlas_texture_path(manifest_path: &Path, atlas_texture: &str) -> Option<PathBuf> {
    let file_name = atlas_texture.rsplit('/').next()?.trim();
    if file_name.is_empty() {
        return None;
    }
    Some(manifest_path.parent()?.join(file_name))
}

static ATLAS_RGBA_CACHE: OnceLock<Mutex<AtlasRgbaCache>> = OnceLock::new();
static NEXT_ATLAS_RGBA_GENERATION: AtomicU64 = AtomicU64::new(1);
