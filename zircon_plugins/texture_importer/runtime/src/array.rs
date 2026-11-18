use std::path::{Path, PathBuf};

use image::RgbaImage;
use serde::Deserialize;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, Texture2DArrayAsset,
    TextureArrayLayerSource, TextureArrayLayout, TextureAsset, TextureAssetDescriptor,
};

use crate::importers::{apply_texture_import_settings, texture_import_outcome};
use crate::manifest_source::{
    decode_manifest_image_bytes, read_manifest_image, resolve_manifest_source,
    validated_relative_source, DecodedManifestImage,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextureArrayManifest {
    #[serde(default)]
    sources: Vec<String>,
    source: Option<String>,
    row_count: Option<u32>,
    row_height: Option<u32>,
}

enum TextureArraySources {
    Files(Vec<String>),
    Sliced {
        source: String,
        layout: TextureArrayLayout,
    },
}

fn parse_array_manifest(
    source_path: &Path,
    source_bytes: &[u8],
) -> Result<TextureArraySources, AssetImportError> {
    let text =
        std::str::from_utf8(source_bytes).map_err(|_| AssetImportError::SourceTextDecode {
            path: source_path.to_owned(),
            source: String::from_utf8(source_bytes.to_vec())
                .expect_err("the borrowed UTF-8 check already rejected these bytes"),
        })?;
    let manifest: TextureArrayManifest = toml::from_str(text).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse texture array manifest {}: {error}",
            source_path.display()
        ))
    })?;
    match (
        manifest.sources.is_empty(),
        manifest.source,
        manifest.row_count,
        manifest.row_height,
    ) {
        (false, None, None, None) => Ok(TextureArraySources::Files(manifest.sources)),
        (true, Some(source), Some(rows), None) => Ok(TextureArraySources::Sliced {
            source,
            layout: TextureArrayLayout::RowCount { rows },
        }),
        (true, Some(source), None, Some(pixels)) => Ok(TextureArraySources::Sliced {
            source,
            layout: TextureArrayLayout::RowHeight { pixels },
        }),
        _ => Err(AssetImportError::Parse(
            "texture array manifest must set either `sources` or one `source` with exactly one of `row_count`/`row_height`"
                .to_string(),
        )),
    }
}

pub(crate) fn texture_array_auxiliary_source_paths(
    source_path: &Path,
    source_bytes: &[u8],
) -> Result<Vec<PathBuf>, AssetImportError> {
    let manifest = parse_array_manifest(source_path, source_bytes)?;
    let sources = match &manifest {
        TextureArraySources::Files(sources) => sources.as_slice(),
        TextureArraySources::Sliced { source, .. } => std::slice::from_ref(source),
    };
    sources
        .iter()
        .map(|source| validated_relative_source(source).map(Path::to_path_buf))
        .collect()
}

pub fn import_texture_array_manifest(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let manifest = parse_array_manifest(&context.source_path, &context.source_bytes)?;
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
    manifest: TextureArraySources,
) -> Result<(Vec<TextureArrayLayerSource>, Vec<TextureAsset>), AssetImportError> {
    match manifest {
        TextureArraySources::Files(sources) => {
            let decoded = sources
                .iter()
                .map(|source| decode_array_image(context, source))
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
        TextureArraySources::Sliced { source, layout } => {
            sliced_layers(decode_array_image(context, &source)?, layout)
        }
    }
}

fn decode_array_image(
    context: &AssetImportContext,
    source: &str,
) -> Result<DecodedManifestImage, AssetImportError> {
    let resolved = resolve_manifest_source(context, source)?;
    match context.captured_auxiliary_source(&resolved.path)? {
        Some(bytes) => decode_manifest_image_bytes(resolved, bytes),
        // Only direct callers without an authoritative capture may read from disk.
        None => read_manifest_image(resolved),
    }
}

fn sliced_layers(
    source: DecodedManifestImage,
    layout: TextureArrayLayout,
) -> Result<(Vec<TextureArrayLayerSource>, Vec<TextureAsset>), AssetImportError> {
    let image_height = source.rgba.height();
    if source.rgba.width() == 0 || image_height == 0 {
        return Err(AssetImportError::Parse(
            "texture array slice source dimensions must be greater than zero".to_string(),
        ));
    }
    let layer_height = match layout {
        TextureArrayLayout::RowCount { rows } if rows > 0 && image_height % rows == 0 => {
            image_height / rows
        }
        TextureArrayLayout::RowHeight { pixels } if pixels > 0 && image_height % pixels == 0 => {
            pixels
        }
        _ => {
            return Err(AssetImportError::Parse(format!(
                "texture array slice must evenly divide image height {image_height}"
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
