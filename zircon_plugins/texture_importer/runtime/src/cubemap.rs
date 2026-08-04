use image::{Rgba, RgbaImage, imageops};
use serde::Deserialize;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, CUBEMAP_FACE_COUNT, CubemapAsset,
    CubemapSourceLayout, TextureAsset, TextureAssetDescriptor,
};
use zircon_runtime::core::framework::render::{
    CubemapFace, RenderImageDimension, cubemap_texel_direction, equirect_uv_from_direction,
};

use crate::importers::{apply_texture_import_settings, texture_import_outcome};
use crate::manifest_source::{DecodedManifestImage, decode_manifest_image};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CubemapManifest {
    layout: CubemapSourceLayout,
    sources: Vec<String>,
}

pub fn import_cubemap_manifest(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let manifest: CubemapManifest = toml::from_str(&context.source_text()?).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse cubemap manifest {}: {error}",
            context.source_path.display()
        ))
    })?;
    let decoded = manifest
        .sources
        .iter()
        .map(|source| decode_manifest_image(context, source))
        .collect::<Result<Vec<_>, _>>()?;
    let references = decoded
        .iter()
        .map(|source| source.reference.clone())
        .collect::<Vec<_>>();
    let faces = decoded_cubemap_faces(manifest.layout, decoded, context)?;
    let asset = CubemapAsset {
        uri: context.uri.clone(),
        descriptor: TextureAssetDescriptor::rgba8_srgb(),
        source_layout: manifest.layout,
        sources: references,
    };
    let texture =
        zircon_runtime::asset::texture_asset_from_cubemap_faces(asset, faces).map_err(|error| {
            AssetImportError::Parse(format!(
                "assemble cubemap manifest {}: {error}",
                context.source_path.display()
            ))
        })?;
    let (texture, diagnostics) = apply_texture_import_settings(context, texture)?;
    Ok(texture_import_outcome(
        context,
        enforce_cube_descriptor(texture),
        diagnostics,
    ))
}

fn decoded_cubemap_faces(
    layout: CubemapSourceLayout,
    decoded: Vec<DecodedManifestImage>,
    context: &AssetImportContext,
) -> Result<Vec<TextureAsset>, AssetImportError> {
    match layout {
        CubemapSourceLayout::SixFiles => decoded
            .into_iter()
            .map(|source| texture_from_image(source.reference.locator, source.rgba))
            .collect(),
        CubemapSourceLayout::HorizontalCross | CubemapSourceLayout::VerticalCross => {
            let [source] = decoded.try_into().map_err(|decoded: Vec<_>| {
                AssetImportError::Parse(format!(
                    "cubemap {:?} layout requires one source, found {}",
                    layout,
                    decoded.len()
                ))
            })?;
            cross_faces(layout, source)
        }
        CubemapSourceLayout::Equirectangular => {
            let [source] = decoded.try_into().map_err(|decoded: Vec<_>| {
                AssetImportError::Parse(format!(
                    "cubemap equirectangular layout requires one source, found {}",
                    decoded.len()
                ))
            })?;
            let face_size = cubemap_face_size(context, source_height(&source));
            equirectangular_faces(source, face_size)
        }
    }
}

fn cross_faces(
    layout: CubemapSourceLayout,
    source: DecodedManifestImage,
) -> Result<Vec<TextureAsset>, AssetImportError> {
    let (columns, rows, offsets, flip_negative_z) = match layout {
        CubemapSourceLayout::HorizontalCross => (
            4,
            3,
            [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)],
            false,
        ),
        CubemapSourceLayout::VerticalCross => {
            (3, 4, [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (1, 3)], true)
        }
        _ => unreachable!("cross_faces receives only cross layouts"),
    };
    if source.rgba.width() % columns != 0 || source.rgba.height() % rows != 0 {
        return Err(AssetImportError::Parse(format!(
            "cubemap {:?} source dimensions {}x{} must be divisible by {}x{} tiles",
            layout,
            source.rgba.width(),
            source.rgba.height(),
            columns,
            rows
        )));
    }
    let face_size = source.rgba.width() / columns;
    if face_size == 0 || source.rgba.height() / rows != face_size {
        return Err(AssetImportError::Parse(format!(
            "cubemap {:?} source dimensions {}x{} do not contain square faces",
            layout,
            source.rgba.width(),
            source.rgba.height()
        )));
    }

    offsets
        .into_iter()
        .enumerate()
        .map(|(face, (column, row))| {
            let mut image = imageops::crop_imm(
                &source.rgba,
                column * face_size,
                row * face_size,
                face_size,
                face_size,
            )
            .to_image();
            if flip_negative_z && face == 5 {
                image = imageops::rotate180(&image);
            }
            texture_from_image(source.reference.locator.clone(), image)
        })
        .collect()
}

fn equirectangular_faces(
    source: DecodedManifestImage,
    face_size: u32,
) -> Result<Vec<TextureAsset>, AssetImportError> {
    if source.rgba.width() != source.rgba.height().saturating_mul(2) {
        return Err(AssetImportError::Parse(format!(
            "equirectangular cubemap source dimensions must use a 2:1 ratio, found {}x{}",
            source.rgba.width(),
            source.rgba.height()
        )));
    }
    CubemapFace::ALL
        .into_iter()
        .map(|face| {
            let image = RgbaImage::from_fn(face_size, face_size, |x, y| {
                let direction = cubemap_texel_direction(face, x, y, face_size);
                let uv = equirect_uv_from_direction(direction);
                sample_equirect_bilinear(&source.rgba, uv)
            });
            texture_from_image(source.reference.locator.clone(), image)
        })
        .collect()
}

fn sample_equirect_bilinear(image: &RgbaImage, uv: [f32; 2]) -> Rgba<u8> {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let x = uv[0].rem_euclid(1.0) * width as f32 - 0.5;
    let y = uv[1].clamp(0.0, 1.0) * height as f32 - 0.5;
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let mut result = [0_u8; 4];
    for (channel, value) in result.iter_mut().enumerate() {
        let sample = |sx: i64, sy: i64| {
            let wrapped_x = sx.rem_euclid(width as i64) as u32;
            let clamped_y = sy.clamp(0, height as i64 - 1) as u32;
            image.get_pixel(wrapped_x, clamped_y)[channel] as f32
        };
        let top = sample(x0, y0) * (1.0 - tx) + sample(x0 + 1, y0) * tx;
        let bottom = sample(x0, y0 + 1) * (1.0 - tx) + sample(x0 + 1, y0 + 1) * tx;
        *value = (top * (1.0 - ty) + bottom * ty).round() as u8;
    }
    Rgba(result)
}

fn texture_from_image(
    uri: zircon_runtime::asset::AssetUri,
    image: RgbaImage,
) -> Result<TextureAsset, AssetImportError> {
    Ok(TextureAsset::new_rgba8(
        uri,
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}

fn enforce_cube_descriptor(texture: TextureAsset) -> TextureAsset {
    let mut descriptor = texture.texture_descriptor();
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = CUBEMAP_FACE_COUNT as u32;
    descriptor.array_layer_count = CUBEMAP_FACE_COUNT as u32;
    texture.with_descriptor(descriptor)
}

fn cubemap_face_size(context: &AssetImportContext, source_height: u32) -> u32 {
    let default = zircon_runtime::core::framework::render::cubemap_face_size_from_equirect_height(
        source_height,
    );
    context
        .import_settings
        .get("cubemap_face_size")
        .and_then(toml::Value::as_integer)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn source_height(source: &DecodedManifestImage) -> u32 {
    source.rgba.height()
}
