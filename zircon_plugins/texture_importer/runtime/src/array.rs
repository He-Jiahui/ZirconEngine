use image::RgbaImage;
use serde::Deserialize;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, Texture2DArrayAsset,
    TextureArrayLayerSource, TextureArrayLayout, TextureAsset, TextureAssetDescriptor,
};

use crate::importers::{apply_texture_import_settings, texture_import_outcome};
use crate::manifest_source::{decode_manifest_image, DecodedManifestImage};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextureArrayManifest {
    #[serde(default)]
    sources: Vec<String>,
    source: Option<String>,
    row_count: Option<u32>,
    row_height: Option<u32>,
}

pub fn import_texture_array_manifest(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let manifest: TextureArrayManifest =
        toml::from_str(&context.source_text()?).map_err(|error| {
            AssetImportError::Parse(format!(
                "parse texture array manifest {}: {error}",
                context.source_path.display()
            ))
        })?;
    let (layer_sources, layers) = resolved_layers(context, manifest)?;
    let asset = Texture2DArrayAsset {
        uri: context.uri.clone(),
        descriptor: TextureAssetDescriptor::rgba8_srgb(),
        layers: layer_sources,
    };
    let texture =
        zircon_runtime::asset::texture_asset_from_array_layers(asset, layers).map_err(|error| {
            AssetImportError::Parse(format!(
                "assemble texture array manifest {}: {error}",
                context.source_path.display()
            ))
        })?;
    let (texture, diagnostics) = apply_texture_import_settings(context, texture)?;
    Ok(texture_import_outcome(context, texture, diagnostics))
}

fn resolved_layers(
    context: &AssetImportContext,
    manifest: TextureArrayManifest,
) -> Result<(Vec<TextureArrayLayerSource>, Vec<TextureAsset>), AssetImportError> {
    match (manifest.sources.is_empty(), manifest.source) {
        (false, None) if manifest.row_count.is_none() && manifest.row_height.is_none() => {
            let decoded = manifest
                .sources
                .iter()
                .map(|source| decode_manifest_image(context, source))
                .collect::<Result<Vec<_>, _>>()?;
            let sources = decoded
                .iter()
                .map(|source| TextureArrayLayerSource::Reference(source.reference.clone()))
                .collect();
            let layers = decoded
                .into_iter()
                .map(texture_from_image)
                .collect::<Vec<_>>();
            Ok((sources, layers))
        }
        (true, Some(source)) if manifest.row_count.is_some() ^ manifest.row_height.is_some() => {
            let decoded = decode_manifest_image(context, &source)?;
            sliced_layers(decoded, manifest.row_count, manifest.row_height)
        }
        _ => Err(AssetImportError::Parse(
            "texture array manifest must set either `sources` or one `source` with exactly one of `row_count`/`row_height`"
                .to_string(),
        )),
    }
}

fn sliced_layers(
    source: DecodedManifestImage,
    row_count: Option<u32>,
    row_height: Option<u32>,
) -> Result<(Vec<TextureArrayLayerSource>, Vec<TextureAsset>), AssetImportError> {
    if source.rgba.width() == 0 || source.rgba.height() == 0 {
        return Err(AssetImportError::Parse(
            "texture array slice source dimensions must be greater than zero".to_string(),
        ));
    }
    let (layout, layer_height) = match (row_count, row_height) {
        (Some(count), None) if count > 0 && source.rgba.height() % count == 0 => (
            TextureArrayLayout::RowCount { rows: count },
            source.rgba.height() / count,
        ),
        (None, Some(height)) if height > 0 && source.rgba.height() % height == 0 => {
            (TextureArrayLayout::RowHeight { pixels: height }, height)
        }
        _ => {
            return Err(AssetImportError::Parse(format!(
                "texture array slice must evenly divide image height {}",
                source.rgba.height()
            )));
        }
    };
    let layers = contiguous_rgba_layer_bytes(&source.rgba, layer_height)
        .map(|rgba| {
            TextureAsset::new_rgba8(
                source.reference.locator.clone(),
                source.rgba.width(),
                layer_height,
                rgba,
            )
        })
        .collect();
    Ok((
        vec![TextureArrayLayerSource::SlicedFromImage {
            reference: source.reference,
            layout,
        }],
        layers,
    ))
}

pub(crate) fn contiguous_rgba_layer_bytes(
    image: &RgbaImage,
    layer_height: u32,
) -> impl Iterator<Item = Vec<u8>> + '_ {
    assert!(
        image.width() > 0
            && layer_height > 0
            && image.height() > 0
            && image.height() % layer_height == 0,
        "contiguous RGBA layers require a positive, evenly divisible height"
    );
    let layer_count = usize::try_from(image.height() / layer_height)
        .expect("u32 texture array layer count fits usize");
    let layer_byte_len = image.as_raw().len() / layer_count;
    image
        .as_raw()
        .chunks_exact(layer_byte_len)
        .map(<[u8]>::to_vec)
}

fn texture_from_image(source: DecodedManifestImage) -> TextureAsset {
    TextureAsset::new_rgba8(
        source.reference.locator,
        source.rgba.width(),
        source.rgba.height(),
        source.rgba.into_raw(),
    )
}
