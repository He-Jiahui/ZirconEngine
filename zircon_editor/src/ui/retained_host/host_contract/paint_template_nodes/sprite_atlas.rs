use std::path::Path;

use super::super::paint_frame::HostPaintAtlasImage;

mod cache;
mod discovery;
mod image;
mod keys;
mod uv;

#[cfg(test)]
use std::fs;
#[cfg(test)]
use zircon_runtime::asset::SpriteAtlasAsset;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use discovery::ATLAS_LIBRARY_DIR;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolve_editor_sprite_atlas_image(
    source_key: &str,
    source_path: &Path,
) -> Option<HostPaintAtlasImage> {
    let entry_name = keys::entry_name_for_source_key(source_key)?;
    for manifest_path in discovery::atlas_manifest_candidates(source_path) {
        let atlas = cache::load_atlas_manifest(&manifest_path)?;
        let entry = atlas
            .entries
            .iter()
            .find(|entry| entry.name == entry_name)?;
        let rgba = image::load_atlas_rgba(&manifest_path, &atlas)?;
        return Some(HostPaintAtlasImage {
            resource_key: atlas.atlas_texture.to_string(),
            width: atlas.width,
            height: atlas.height,
            rgba: Some(rgba),
            uv: uv::host_uv_rect(entry.uv_rect),
        });
    }
    None
}

#[cfg(test)]
#[path = "sprite_atlas_tests.rs"]
mod tests;
