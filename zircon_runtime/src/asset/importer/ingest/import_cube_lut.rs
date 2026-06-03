use crate::asset::assets::{texture_asset_from_cube_lut, ImportedAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_cube_lut(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source = std::str::from_utf8(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode cube LUT {} as UTF-8: {error}",
            context.source_path.display()
        ))
    })?;
    let texture = texture_asset_from_cube_lut(context.uri.clone(), source).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse cube LUT {}: {error}",
            context.source_path.display()
        ))
    })?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}
