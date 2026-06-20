use std::path::{Path, PathBuf};

use zircon_runtime::asset::{AssetUri, SpriteAtlasAsset};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_atlas_rgba(
    manifest_path: &Path,
    atlas: &SpriteAtlasAsset,
) -> Option<Vec<u8>> {
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
