use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use zircon_runtime::asset::{SpriteAtlasAsset, validate_sprite_atlas_asset};

const MAX_ATLAS_MANIFEST_CACHE_ENTRIES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AtlasCacheKey {
    path: PathBuf,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_atlas_manifest(
    path: &Path,
) -> Option<SpriteAtlasAsset> {
    let key = AtlasCacheKey::from_path(path);
    let cache = ATLAS_MANIFEST_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(cached) = cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(&key)
    {
        return cached.clone();
    }

    let atlas = fs::read_to_string(path)
        .ok()
        .and_then(|document| toml::from_str::<SpriteAtlasAsset>(&document).ok())
        .filter(|atlas| validate_sprite_atlas_asset(atlas).is_ok());
    let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    if !cache.contains_key(&key) && cache.len() >= MAX_ATLAS_MANIFEST_CACHE_ENTRIES {
        if let Some(evicted_key) = cache.keys().next().cloned() {
            cache.remove(&evicted_key);
        }
    }
    cache.insert(key, atlas.clone());
    atlas
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn clear_atlas_manifest_cache()
 {
    if let Some(cache) = ATLAS_MANIFEST_CACHE.get() {
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clear();
    }
}

impl AtlasCacheKey {
    fn from_path(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
}

static ATLAS_MANIFEST_CACHE: OnceLock<Mutex<BTreeMap<AtlasCacheKey, Option<SpriteAtlasAsset>>>> =
    OnceLock::new();
