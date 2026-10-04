use std::fs;
use std::path::PathBuf;

use image::{ColorType, ImageFormat};
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{AssetUri, SpriteAtlasAsset};

use super::config::SpriteAtlasBuildConfig;
use super::packer::{
    atlas_manifest_uri, atlas_texture_uri, PackedSpriteAtlas, SpriteAtlasBuildError,
};

const ATLAS_CACHE_DIR: &str = "editor-sprite-atlases";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteAtlasArtifactReport {
    pub atlas_texture: AssetUri,
    pub manifest: AssetUri,
    pub image_path: PathBuf,
    pub manifest_path: PathBuf,
}

pub fn write_sprite_atlas_artifacts(
    project: &ProjectManager,
    config: &SpriteAtlasBuildConfig,
    packed: &PackedSpriteAtlas,
) -> Result<SpriteAtlasArtifactReport, SpriteAtlasBuildError> {
    config
        .validate()
        .map_err(SpriteAtlasBuildError::InvalidConfig)?;

    let artifact_root = project.paths().cache_root().join(ATLAS_CACHE_DIR);
    fs::create_dir_all(&artifact_root)?;

    let atlas_texture = atlas_texture_uri(config)?;
    let manifest = atlas_manifest_uri(config)?;
    let image_path = artifact_root.join(format!("{}.png", config.output_stem));
    let manifest_path = artifact_root.join(format!("{}.toml", config.output_stem));

    image::save_buffer_with_format(
        &image_path,
        &packed.rgba,
        packed.atlas.width,
        packed.atlas.height,
        ColorType::Rgba8,
        ImageFormat::Png,
    )
    .map_err(SpriteAtlasBuildError::from)?;

    let manifest_asset = SpriteAtlasAsset {
        atlas_texture: atlas_texture.clone(),
        width: packed.atlas.width,
        height: packed.atlas.height,
        padding: packed.atlas.padding,
        entries: packed.atlas.entries.clone(),
    };
    zircon_runtime::asset::validate_sprite_atlas_asset(&manifest_asset)
        .map_err(|error| SpriteAtlasBuildError::AtlasValidation(error.to_string()))?;
    let document = toml::to_string_pretty(&manifest_asset)?;
    fs::write(&manifest_path, document)?;

    Ok(SpriteAtlasArtifactReport {
        atlas_texture,
        manifest,
        image_path,
        manifest_path,
    })
}

#[cfg(test)]
#[path = "tests/artifact.rs"]
mod tests;
