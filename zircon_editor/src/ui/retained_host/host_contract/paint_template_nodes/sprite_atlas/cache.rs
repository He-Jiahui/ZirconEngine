use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use zircon_runtime::asset::{validate_sprite_atlas_asset, SpriteAtlasAsset};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AtlasCacheKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
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
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, atlas.clone());
    atlas
}

impl AtlasCacheKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(fs::Metadata::len);
        Self {
            path,
            modified_unix_ns,
            len,
        }
    }
}

static ATLAS_MANIFEST_CACHE: OnceLock<Mutex<BTreeMap<AtlasCacheKey, Option<SpriteAtlasAsset>>>> =
    OnceLock::new();
