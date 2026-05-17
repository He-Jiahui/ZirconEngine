use crate::asset::assets::{ImportedAsset, TextureAsset};
use crate::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
};

pub(crate) fn import_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let texture =
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba)
            .with_import_settings(&context.import_settings)
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "apply texture import settings {}: {error}",
                    context.source_path.display()
                ))
            })?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}
