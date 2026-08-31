use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::super::paint_frame::{HostPaintAtlasImage, HostPaintImageUvRect};

mod cache;
mod discovery;
mod image;
mod keys;
mod uv;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use discovery::ATLAS_CACHE_DIR;

const MAX_ATLAS_RESOLUTION_CACHE_ENTRIES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AtlasResolutionCacheKey {
    source_key: String,
    source_path: PathBuf,
}

#[derive(Clone)]
struct AtlasResolution {
    manifest_path: PathBuf,
    resource_key: String,
    width: u32,
    height: u32,
    uv: HostPaintImageUvRect,
}

pub(crate) fn invalidate_editor_sprite_atlas_cache() {
    if let Some(cache) = ATLAS_RESOLUTION_CACHE.get() {
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clear();
    }
    cache::clear_atlas_manifest_cache();
    image::clear_atlas_rgba_cache();
}

pub(crate) fn copy_editor_sprite_atlas_rgba(
    resource_key: &str,
    generation: u64,
) -> Option<Vec<u8>> {
    image::copy_atlas_rgba(resource_key, generation)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolve_editor_sprite_atlas_image(
    source_key: &str,
    source_path: &Path,
) -> Option<HostPaintAtlasImage> {
    let resolved = resolve_atlas(source_key, source_path)?;
    let decoded = image::load_atlas_rgba(
        &resolved.manifest_path,
        &resolved.resource_key,
        resolved.width,
        resolved.height,
    )?;
    Some(HostPaintAtlasImage {
        resource_key: resolved.resource_key,
        resource_generation: decoded.generation,
        width: resolved.width,
        height: resolved.height,
        rgba: None,
        uv: resolved.uv,
    })
}

fn resolve_atlas(source_key: &str, source_path: &Path) -> Option<AtlasResolution> {
    let key = AtlasResolutionCacheKey {
        source_key: source_key.to_string(),
        source_path: source_path.to_path_buf(),
    };
    let cache = ATLAS_RESOLUTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(cached) = cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(&key)
    {
        return cached.clone();
    }
    let entry_name = keys::entry_name_for_source_key(source_key)?;
    let resolved = resolve_atlas_uncached(&entry_name, source_path);
    let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    insert_cached_resolution(&mut cache, key, resolved.clone());
    resolved
}

fn insert_cached_resolution(
    cache: &mut HashMap<AtlasResolutionCacheKey, Option<AtlasResolution>>,
    key: AtlasResolutionCacheKey,
    resolved: Option<AtlasResolution>,
) {
    if !cache.contains_key(&key) && cache.len() >= MAX_ATLAS_RESOLUTION_CACHE_ENTRIES {
        if let Some(evicted_key) = cache.keys().min().cloned() {
            cache.remove(&evicted_key);
        }
    }
    cache.insert(key, resolved);
}

fn resolve_atlas_uncached(entry_name: &str, source_path: &Path) -> Option<AtlasResolution> {
    for manifest_path in discovery::atlas_manifest_candidates(source_path) {
        let atlas = cache::load_atlas_manifest(&manifest_path)?;
        let entry = atlas
            .entries
            .iter()
            .find(|entry| entry.name == entry_name)?;
        return Some(AtlasResolution {
            manifest_path,
            resource_key: atlas.atlas_texture.to_string(),
            width: atlas.width,
            height: atlas.height,
            uv: uv::host_uv_rect(entry.uv_rect),
        });
    }
    None
}

static ATLAS_RESOLUTION_CACHE: OnceLock<
    Mutex<HashMap<AtlasResolutionCacheKey, Option<AtlasResolution>>>,
> = OnceLock::new();

#[cfg(test)]
#[path = "sprite_atlas_tests/mod.rs"]
mod tests;
