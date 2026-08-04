mod kernel;

use zircon_runtime::asset::{AssetImportError, TextureAsset, TexturePayload};
use zircon_runtime::core::framework::render::{RenderImageDimension, TextureMipPolicy};

use self::kernel::downsample_rgba8;

const RGBA8_TEXEL_SIZE: usize = 4;

pub(crate) fn generate_offline_mips(
    mut texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    let mut descriptor = texture.texture_descriptor();
    if descriptor.metadata.mip_policy != TextureMipPolicy::GenerateOffline
        || !matches!(&texture.payload, TexturePayload::Rgba8)
    {
        return Ok(texture);
    }
    if descriptor.dimension == RenderImageDimension::D3 {
        return Err(AssetImportError::Parse(format!(
            "offline mip generation does not support 3d rgba8 texture {}",
            texture.uri
        )));
    }
    if texture.width == 0 || texture.height == 0 {
        return Err(AssetImportError::Parse(format!(
            "offline mip generation requires non-zero rgba8 dimensions for {}",
            texture.uri
        )));
    }

    let layer_count = descriptor.depth_or_array_layers.max(1);
    let base_layer_len = rgba8_level_len(texture.width, texture.height).ok_or_else(|| {
        AssetImportError::Parse(format!(
            "offline mip generation dimensions overflow for {}",
            texture.uri
        ))
    })?;
    let base_len = base_layer_len
        .checked_mul(layer_count as usize)
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "offline mip generation layer size overflows for {}",
                texture.uri
            ))
        })?;
    if texture.rgba.len() != base_len {
        return Err(AssetImportError::Parse(format!(
            "offline mip generation expects base-level rgba8 payload of {base_len} bytes for {}, found {}",
            texture.uri,
            texture.rgba.len()
        )));
    }

    let mip_count = full_mip_count(texture.width, texture.height);
    let total_len = rgba8_mip_chain_len(texture.width, texture.height, mip_count, layer_count)
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "offline mip generation chain size overflows for {}",
                texture.uri
            ))
        })?;
    let mut packed_mips = Vec::with_capacity(total_len);
    packed_mips.extend_from_slice(&texture.rgba);
    let mut current_layers = texture
        .rgba
        .chunks_exact(base_layer_len)
        .map(Vec::from)
        .collect::<Vec<_>>();
    let mut current_width = texture.width;
    let mut current_height = texture.height;

    // The uploader consumes every mip level with all array/cube layers packed contiguously.
    while current_width > 1 || current_height > 1 {
        let next_width = (current_width / 2).max(1);
        let next_height = (current_height / 2).max(1);
        let next_layers = current_layers
            .iter()
            .map(|source| {
                downsample_rgba8(
                    source,
                    current_width,
                    current_height,
                    descriptor.metadata.color_space,
                    descriptor.metadata.usage_hint,
                    descriptor.metadata.mip_filter,
                )
                .ok_or_else(|| {
                    AssetImportError::Parse(format!(
                        "offline mip generation target dimensions overflow for {}",
                        texture.uri
                    ))
                })
            })
            .collect::<Result<Vec<_>, AssetImportError>>()?;
        for layer in &next_layers {
            packed_mips.extend_from_slice(layer);
        }
        current_layers = next_layers;
        current_width = next_width;
        current_height = next_height;
    }

    debug_assert_eq!(packed_mips.len(), total_len);
    descriptor.mip_count = mip_count;
    texture.rgba = packed_mips;
    texture.descriptor = Some(descriptor);
    Ok(texture)
}

pub(crate) fn prepare_runtime_mips(
    mut texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    let mut descriptor = texture.texture_descriptor();
    if descriptor.metadata.mip_policy != TextureMipPolicy::GenerateRuntime {
        return Ok(texture);
    }
    if !matches!(&texture.payload, TexturePayload::Rgba8) {
        return Err(AssetImportError::Parse(format!(
            "runtime mip generation requires an uncompressed rgba8 payload for {}",
            texture.uri
        )));
    }
    if !matches!(
        descriptor.dimension,
        RenderImageDimension::D2 | RenderImageDimension::Cube
    ) {
        return Err(AssetImportError::Parse(format!(
            "runtime mip generation supports only 2d or cube rgba8 texture {}",
            texture.uri
        )));
    }
    if texture.width == 0 || texture.height == 0 {
        return Err(AssetImportError::Parse(format!(
            "runtime mip generation requires non-zero rgba8 dimensions for {}",
            texture.uri
        )));
    }

    let layer_count = descriptor.depth_or_array_layers.max(1);
    let expected_base_len = rgba8_level_len(texture.width, texture.height)
        .and_then(|level_len| level_len.checked_mul(layer_count as usize))
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "runtime mip generation dimensions overflow for {}",
                texture.uri
            ))
        })?;
    if texture.rgba.len() != expected_base_len {
        return Err(AssetImportError::Parse(format!(
            "runtime mip generation expects base-level rgba8 payload of {expected_base_len} bytes for {}, found {}",
            texture.uri,
            texture.rgba.len()
        )));
    }

    descriptor.mip_count = full_mip_count(texture.width, texture.height);
    texture.descriptor = Some(descriptor);
    Ok(texture)
}

fn full_mip_count(mut width: u32, mut height: u32) -> u32 {
    let mut count = 1;
    while width > 1 || height > 1 {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
        count += 1;
    }
    count
}

fn rgba8_mip_chain_len(width: u32, height: u32, mip_count: u32, layer_count: u32) -> Option<usize> {
    (0..mip_count).try_fold(0_usize, |total, level| {
        let level_len = rgba8_level_len(mip_extent(width, level), mip_extent(height, level))?;
        total.checked_add(level_len.checked_mul(layer_count as usize)?)
    })
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    if level >= u32::BITS {
        1
    } else {
        (value >> level).max(1)
    }
}

fn rgba8_level_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(RGBA8_TEXEL_SIZE)
}
