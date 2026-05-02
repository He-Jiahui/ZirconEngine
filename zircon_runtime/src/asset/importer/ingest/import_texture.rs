use image::GenericImageView;

use crate::asset::assets::{ImportedAsset, TextureAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, TexturePayload};

pub(crate) fn import_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let image = image::load_from_memory(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode image {}: {error}",
            context.source_path.display()
        ))
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    Ok(AssetImportOutcome::new(ImportedAsset::Texture(
        TextureAsset {
            uri: context.uri.clone(),
            width,
            height,
            rgba: rgba.into_raw(),
            payload: TexturePayload::Rgba8,
        },
    )))
}
