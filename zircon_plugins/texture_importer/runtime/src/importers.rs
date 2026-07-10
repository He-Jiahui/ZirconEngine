use crate::container::parse_container_info;
use zircon_runtime::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
    ImportedAsset, TextureAsset, TextureAssetDescriptor,
};

pub fn import_image(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let texture = apply_texture_import_settings(
        context,
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba),
    )?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

pub fn import_psd(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let psd = psd::Psd::from_bytes(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode psd {}: {error}",
            context.source_path.display()
        ))
    })?;
    let width = psd.width();
    let height = psd.height();
    let rgba = psd.rgba();
    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return Err(AssetImportError::Parse(format!(
            "decode psd {}: decoded rgba length {} did not match expected {}",
            context.source_path.display(),
            rgba.len(),
            expected_len
        )));
    }

    let texture = apply_texture_import_settings(
        context,
        TextureAsset::new_rgba8(context.uri.clone(), width, height, rgba),
    )?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

pub fn import_texture_container(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let info = parse_container_info(context)?;
    let mut descriptor =
        TextureAssetDescriptor::container(info.format.clone(), info.mip_count, info.array_layers);
    descriptor.dimension = info.dimension;
    descriptor.depth_or_array_layers = info.depth_or_array_layers;
    let texture = apply_texture_import_settings(
        context,
        TextureAsset::new_container(
            context.uri.clone(),
            info.width,
            info.height,
            info.format,
            context.source_bytes.clone(),
            info.mip_count,
            info.array_layers,
        )
        .with_descriptor(descriptor),
    )?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

pub(crate) fn apply_texture_import_settings(
    context: &AssetImportContext,
    texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    texture
        .apply_import_settings(&context.import_settings)
        .map_err(|error| {
            AssetImportError::Parse(format!(
                "apply texture import settings {}: {error}",
                context.source_path.display()
            ))
        })
}
