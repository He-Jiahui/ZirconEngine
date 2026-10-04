use gltf::image::{Data as GltfImageData, Format as GltfImageFormat};

use crate::asset::assets::{build_decoded_rgba8_texture, TextureAssetDescriptor};
use crate::asset::{
    AssetImportError, AssetImportOutcome, AssetUri, ImportedAsset, ImportedAssetEntry,
};
use crate::core::framework::render::{
    RenderSamplerAddressMode, RenderSamplerDescriptor, RenderSamplerFilter, TextureMipPolicy,
    TextureNormalConvention,
};

use super::{gltf_texture_color_space_usages, gltf_texture_label, GltfTextureVariant};

pub fn validate_gltf_texture_import_support(
    document: &gltf::Document,
) -> Result<(), AssetImportError> {
    if let Some(texture) = document
        .textures()
        .find(|texture| texture.extension_value("KHR_texture_basisu").is_some())
    {
        return Err(AssetImportError::Parse(format!(
            "gltf texture {} uses unsupported KHR_texture_basisu; a KTX2/BasisU transcoder is required",
            texture.index()
        )));
    }
    Ok(())
}

pub fn add_gltf_texture_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
    images: Vec<GltfImageData>,
) -> Result<AssetImportOutcome, AssetImportError> {
    validate_gltf_texture_import_support(document)?;
    let texture_usages = gltf_texture_color_space_usages(document);
    let texture_sources = document
        .textures()
        .map(|texture| gltf_texture_source_index(&texture))
        .collect::<Result<Vec<_>, _>>()?;
    let mut remaining_uses = vec![0usize; images.len()];
    for &image_index in &texture_sources {
        let uses = remaining_uses.get_mut(image_index).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf texture references missing image {image_index}"
            ))
        })?;
        *uses += 1;
    }
    let mut images = images.into_iter().map(Some).collect::<Vec<_>>();

    for (texture, image_index) in document.textures().zip(texture_sources) {
        let uses = remaining_uses.get_mut(image_index).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf texture {} references missing image {}",
                texture.index(),
                image_index
            ))
        })?;
        *uses -= 1;
        let image = if *uses == 0 {
            images[image_index].take().expect("validated image source")
        } else {
            images[image_index]
                .as_ref()
                .expect("validated image source")
                .clone()
        };
        let (width, height) = (image.width, image.height);
        let rgba = rgba8_pixels_from_gltf_image(image, image_index)?;
        let variants = texture_usages
            .get(texture.index())
            .copied()
            .unwrap_or_default()
            .texture_variants();
        let variant_count = variants.len();
        let mut rgba = Some(rgba);
        for (variant_index, variant) in variants.into_iter().enumerate() {
            let uri = gltf_label_uri(
                root_uri,
                &gltf_texture_label(texture.index(), variant, &texture_usages),
            );
            let pixels = if variant_index + 1 == variant_count {
                rgba.take()
                    .expect("last glTF texture variant owns decoded pixels")
            } else {
                rgba.as_ref()
                    .expect("glTF texture variants retain decoded pixels until the final variant")
                    .clone()
            };
            let asset = build_decoded_rgba8_texture(
                uri.clone(),
                width,
                height,
                pixels,
                gltf_texture_descriptor(&texture, variant),
            )
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "build gltf texture {} decoded rgba8 payload: {error}",
                    texture.index()
                ))
            })?;
            outcome = outcome
                .with_dependency(uri.clone())
                .with_entry(ImportedAssetEntry::new(uri, ImportedAsset::Texture(asset)));
        }
    }
    Ok(outcome)
}

fn gltf_texture_descriptor(
    texture: &gltf::Texture<'_>,
    variant: GltfTextureVariant,
) -> TextureAssetDescriptor {
    let mut descriptor =
        TextureAssetDescriptor::decoded_rgba8_for_import_usage(variant.usage_hint());
    if variant.usage_hint() == crate::core::framework::render::TextureUsageHint::Normal {
        descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceGl;
    }
    descriptor.metadata.mip_policy = if gltf_sampler_uses_mipmaps(texture) {
        TextureMipPolicy::GenerateOffline
    } else {
        TextureMipPolicy::None
    };
    descriptor.sampler = gltf_sampler_descriptor(texture);
    descriptor
}

fn gltf_sampler_uses_mipmaps(texture: &gltf::Texture<'_>) -> bool {
    !matches!(
        texture.sampler().min_filter(),
        Some(gltf::texture::MinFilter::Nearest | gltf::texture::MinFilter::Linear)
    )
}

fn gltf_sampler_descriptor(texture: &gltf::Texture<'_>) -> RenderSamplerDescriptor {
    let sampler = texture.sampler();
    let mut descriptor = RenderSamplerDescriptor {
        address_mode_u: gltf_sampler_address_mode(sampler.wrap_s()),
        address_mode_v: gltf_sampler_address_mode(sampler.wrap_t()),
        // glTF textures are 2D; repeat preserves its default coordinate policy for unused W.
        address_mode_w: RenderSamplerAddressMode::Repeat,
        ..RenderSamplerDescriptor::default()
    };
    if let Some(filter) = sampler.mag_filter() {
        descriptor.mag_filter = gltf_mag_filter(filter);
    }
    if let Some(filter) = sampler.min_filter() {
        (descriptor.min_filter, descriptor.mipmap_filter) = gltf_min_filter(filter);
    }
    descriptor
}

fn gltf_sampler_address_mode(mode: gltf::texture::WrappingMode) -> RenderSamplerAddressMode {
    match mode {
        gltf::texture::WrappingMode::ClampToEdge => RenderSamplerAddressMode::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => RenderSamplerAddressMode::MirrorRepeat,
        gltf::texture::WrappingMode::Repeat => RenderSamplerAddressMode::Repeat,
    }
}

fn gltf_mag_filter(filter: gltf::texture::MagFilter) -> RenderSamplerFilter {
    match filter {
        gltf::texture::MagFilter::Nearest => RenderSamplerFilter::Nearest,
        gltf::texture::MagFilter::Linear => RenderSamplerFilter::Linear,
    }
}

fn gltf_min_filter(filter: gltf::texture::MinFilter) -> (RenderSamplerFilter, RenderSamplerFilter) {
    match filter {
        gltf::texture::MinFilter::Nearest | gltf::texture::MinFilter::NearestMipmapNearest => {
            (RenderSamplerFilter::Nearest, RenderSamplerFilter::Nearest)
        }
        gltf::texture::MinFilter::Linear | gltf::texture::MinFilter::LinearMipmapNearest => {
            (RenderSamplerFilter::Linear, RenderSamplerFilter::Nearest)
        }
        gltf::texture::MinFilter::NearestMipmapLinear => {
            (RenderSamplerFilter::Nearest, RenderSamplerFilter::Linear)
        }
        gltf::texture::MinFilter::LinearMipmapLinear => {
            (RenderSamplerFilter::Linear, RenderSamplerFilter::Linear)
        }
    }
}

fn gltf_texture_source_index(texture: &gltf::Texture<'_>) -> Result<usize, AssetImportError> {
    if let Some(extension) = texture.extension_value("EXT_texture_webp") {
        let source = extension
            .get("source")
            .and_then(serde_json::Value::as_u64)
            .and_then(|source| usize::try_from(source).ok())
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "gltf texture {} has malformed EXT_texture_webp source metadata",
                    texture.index()
                ))
            })?;
        return Ok(source);
    }
    texture.source().map(|image| image.index()).ok_or_else(|| {
        AssetImportError::Parse(format!(
            "gltf texture {} has neither a core source nor EXT_texture_webp source",
            texture.index()
        ))
    })
}

fn rgba8_pixels_from_gltf_image(
    image: GltfImageData,
    image_index: usize,
) -> Result<Vec<u8>, AssetImportError> {
    let pixel_count = image
        .width
        .checked_mul(image.height)
        .and_then(|pixels| usize::try_from(pixels).ok())
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf image {image_index} extent {}x{} is too large",
                image.width, image.height
            ))
        })?;

    if image.format == GltfImageFormat::R8G8B8A8 {
        validate_image_len(&image, image_index, pixel_count * 4)?;
        return Ok(image.pixels);
    }

    let mut rgba = Vec::with_capacity(pixel_count * 4);
    match image.format {
        GltfImageFormat::R8 => {
            validate_image_len(&image, image_index, pixel_count)?;
            for value in &image.pixels {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        GltfImageFormat::R8G8 => {
            validate_image_len(&image, image_index, pixel_count * 2)?;
            for chunk in image.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
        }
        GltfImageFormat::R8G8B8 => {
            validate_image_len(&image, image_index, pixel_count * 3)?;
            for chunk in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
        }
        GltfImageFormat::R8G8B8A8 => unreachable!("RGBA8 returned above"),
        other => {
            return Err(AssetImportError::Parse(format!(
                "gltf image {image_index} format {other:?} is not supported for TextureAsset rgba8 output"
            )));
        }
    }
    Ok(rgba)
}

fn validate_image_len(
    image: &GltfImageData,
    image_index: usize,
    expected: usize,
) -> Result<(), AssetImportError> {
    if image.pixels.len() != expected {
        return Err(AssetImportError::Parse(format!(
            "gltf image {image_index} expected {expected} decoded bytes but found {}",
            image.pixels.len()
        )));
    }
    Ok(())
}

fn gltf_label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}"))
        .expect("generated gltf subasset locator must be valid")
}

#[cfg(test)]
#[path = "tests/gltf_texture_subassets.rs"]
mod tests;
