use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use zircon_runtime::asset::{
    validate_sprite_atlas_asset, AssetUri, SpriteAtlasAsset, SpriteAtlasUvRect,
};

use super::super::paint_frame::{HostPaintAtlasImage, HostPaintImageUvRect};

const ATLAS_LIBRARY_DIR: &str = "editor-sprite-atlases";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AtlasCacheKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

pub(super) fn resolve_editor_sprite_atlas_image(
    source_key: &str,
    source_path: &Path,
) -> Option<HostPaintAtlasImage> {
    let entry_name = entry_name_for_source_key(source_key)?;
    for manifest_path in atlas_manifest_candidates(source_path) {
        let atlas = load_atlas_manifest(&manifest_path)?;
        let entry = atlas
            .entries
            .iter()
            .find(|entry| entry.name == entry_name)?;
        let rgba = load_atlas_rgba(&manifest_path, &atlas)?;
        return Some(HostPaintAtlasImage {
            resource_key: atlas.atlas_texture.to_string(),
            width: atlas.width,
            height: atlas.height,
            rgba: Some(rgba),
            uv: host_uv_rect(entry.uv_rect),
        });
    }
    None
}

fn entry_name_for_source_key(source_key: &str) -> Option<&str> {
    source_key
        .strip_prefix("template-image:")
        .or_else(|| source_key.strip_prefix("template-icon:"))
        .or_else(|| source_key.strip_prefix("image:"))
        .or_else(|| source_key.strip_prefix("icon:"))
}

fn atlas_manifest_candidates(source_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let Some(root) = source_path
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "assets"))
    else {
        return candidates;
    };
    push_candidate(
        &mut candidates,
        root.join("library").join(ATLAS_LIBRARY_DIR),
    );
    if let Some(parent) = root.parent() {
        push_candidate(
            &mut candidates,
            parent.join("library").join(ATLAS_LIBRARY_DIR),
        );
    }
    candidates
}

fn push_candidate(candidates: &mut Vec<PathBuf>, atlas_dir: PathBuf) {
    let Ok(entries) = fs::read_dir(atlas_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
            && !candidates.iter().any(|candidate| candidate == &path)
        {
            candidates.push(path);
        }
    }
    candidates.sort();
}

fn load_atlas_manifest(path: &Path) -> Option<SpriteAtlasAsset> {
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

fn load_atlas_rgba(manifest_path: &Path, atlas: &SpriteAtlasAsset) -> Option<Vec<u8>> {
    let texture_path = atlas_texture_path(manifest_path, &atlas.atlas_texture)?;
    let image = image::open(texture_path).ok()?.into_rgba8();
    (image.dimensions() == (atlas.width, atlas.height)).then(|| image.into_raw())
}

fn atlas_texture_path(manifest_path: &Path, atlas_texture: &AssetUri) -> Option<PathBuf> {
    let texture = atlas_texture.to_string();
    let file_name = texture.rsplit('/').next()?.trim();
    if file_name.is_empty() {
        return None;
    }
    Some(manifest_path.parent()?.join(file_name))
}

fn host_uv_rect(uv: SpriteAtlasUvRect) -> HostPaintImageUvRect {
    HostPaintImageUvRect {
        min: uv.min,
        max: uv.max,
    }
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

#[cfg(test)]
#[path = "sprite_atlas_tests.rs"]
mod tests;
