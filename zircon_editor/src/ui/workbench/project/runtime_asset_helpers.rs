use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::assets::{AlphaMode, MaterialAsset};
use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::resource::ResourceLocator;
use zircon_runtime::scene::world::SceneProjectError;

use super::constants::DEFAULT_SHADER_URI;

pub(in crate::ui::workbench::project) fn write_if_missing(
    path: PathBuf,
    contents: impl AsRef<[u8]>,
) -> Result<(), SceneProjectError> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)?;
    Ok(())
}

pub(in crate::ui::workbench::project) fn default_material_asset(
) -> Result<MaterialAsset, SceneProjectError> {
    Ok(MaterialAsset {
        name: Some("Default".to_string()),
        shader: AssetReference::from_locator(
            ResourceLocator::parse(DEFAULT_SHADER_URI).expect("default shader uri"),
        ),
        base_color: [0.85, 0.85, 0.85, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    })
}

pub(in crate::ui::workbench::project) fn parse_asset_uri(
    value: &str,
) -> Result<ResourceLocator, SceneProjectError> {
    ResourceLocator::parse(value)
        .map_err(|error| SceneProjectError::Asset(AssetImportError::from(error)))
}

pub(in crate::ui::workbench::project) fn invalid_data(
    message: impl Into<String>,
) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into())
}
