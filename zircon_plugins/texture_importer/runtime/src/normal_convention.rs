use zircon_runtime::asset::{
    normalize_texture_normal_map_convention, AssetImportError, TextureAsset,
};

/// Converts decoded normal maps into the engine-wide right-handed tangent-space GL convention.
pub(crate) fn normalize_normal_map_convention(
    texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    normalize_texture_normal_map_convention(texture)
        .map_err(|error| AssetImportError::Parse(error.to_string()))
}

#[cfg(test)]
#[path = "tests/normal_convention.rs"]
mod tests;
