use zircon_runtime::asset::{AssetImportError, TextureAsset, TexturePayload};
use zircon_runtime::core::framework::render::{
    RenderImageDimension, TextureCompressionTarget, TextureNormalConvention, TextureUsageHint,
};

const DDS_CLASSIC_HEADER_LEN: usize = 128;
const DDS_DX10_HEADER_LEN: usize = DDS_CLASSIC_HEADER_LEN + 20;
const DDS_FLAGS: u32 = 0x0008_1007;
const DDS_FLAGS_WITH_MIPS: u32 = DDS_FLAGS | 0x0002_0000;
const DDS_PIXEL_FORMAT_FOURCC: u32 = 0x0000_0004;
const DDS_CAPS_TEXTURE: u32 = 0x0000_1000;
const DDS_CAPS_COMPLEX: u32 = 0x0000_0008;
const DDS_CAPS_MIPMAP: u32 = 0x0040_0000;
const DDS_CAPS2_CUBEMAP: u32 = 0x0000_0200;
const DDS_CAPS2_CUBEMAP_ALL_FACES: u32 = DDS_CAPS2_CUBEMAP
    | 0x0000_0400
    | 0x0000_0800
    | 0x0000_1000
    | 0x0000_2000
    | 0x0000_4000
    | 0x0000_8000;
const DDS_FOURCC_ATI2: u32 = u32::from_le_bytes(*b"ATI2");
const DDS_FOURCC_DX10: u32 = u32::from_le_bytes(*b"DX10");
const DXGI_FORMAT_BC5_UNORM: u32 = 83;
const DDS_RESOURCE_DIMENSION_TEXTURE2D: u32 = 3;
const BC5_BLOCK_EDGE: u32 = 4;
const BC5_BLOCK_BYTES: usize = 16;

pub(crate) fn transcode_normal_bc5(
    mut texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    let mut descriptor = texture.texture_descriptor();
    if descriptor.metadata.usage_hint != TextureUsageHint::Normal
        || descriptor.metadata.compression != TextureCompressionTarget::Bc5
    {
        return Ok(texture);
    }
    if descriptor.metadata.normal_convention != TextureNormalConvention::TangentSpaceGl {
        return Err(AssetImportError::Parse(format!(
            "bc5 normal transcode requires canonical tangent-space GL input: {}",
            texture.uri
        )));
    }
    if !matches!(&texture.payload, TexturePayload::Rgba8) {
        // Container payloads can already be hardware-compressed and have no Rust encoder path.
        // Preserve them instead of rejecting an otherwise upload-ready imported asset.
        return Ok(texture);
    }
    if !matches!(
        descriptor.dimension,
        RenderImageDimension::D2 | RenderImageDimension::Cube
    ) {
        return Err(AssetImportError::Parse(format!(
            "bc5 normal transcode supports only 2d or cube textures: {}",
            texture.uri
        )));
    }

    let layer_count = descriptor.depth_or_array_layers.max(1);
    if descriptor.dimension == RenderImageDimension::Cube && layer_count != 6 {
        return Err(AssetImportError::Parse(format!(
            "bc5 cubemap normal transcode requires six faces: {}",
            texture.uri
        )));
    }

    let mip_count = descriptor.mip_count.max(1);
    let expected_source_len =
        rgba_mip_chain_len(texture.width, texture.height, mip_count, layer_count).ok_or_else(
            || {
                AssetImportError::Parse(format!(
                    "bc5 normal transcode dimensions overflow for {}",
                    texture.uri
                ))
            },
        )?;
    if texture.rgba.len() != expected_source_len {
        return Err(AssetImportError::Parse(format!(
            "bc5 normal transcode expects {expected_source_len} rgba8 bytes for {}, found {}",
            texture.uri,
            texture.rgba.len()
        )));
    }

    let payload = encode_bc5_payload(
        &texture.rgba,
        texture.width,
        texture.height,
        mip_count,
        layer_count,
    )
    .ok_or_else(|| {
        AssetImportError::Parse(format!(
            "bc5 normal transcode output size overflows for {}",
            texture.uri
        ))
    })?;
    let base_level_byte_size =
        u32::try_from(bc5_level_len(texture.width, texture.height).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "bc5 normal transcode base level size overflows for {}",
                texture.uri
            ))
        })?)
        .map_err(|_| {
            AssetImportError::Parse(format!(
                "bc5 normal transcode base level exceeds DDS 32-bit linear size for {}",
                texture.uri
            ))
        })?;
    let (format, header) = if descriptor.dimension == RenderImageDimension::Cube {
        (
            "dds/ati2",
            encode_classic_dds_header(
                texture.width,
                texture.height,
                mip_count,
                base_level_byte_size,
                true,
            ),
        )
    } else if layer_count > 1 {
        (
            "dds/dxgi-83",
            encode_dx10_array_dds_header(
                texture.width,
                texture.height,
                mip_count,
                base_level_byte_size,
                layer_count,
            ),
        )
    } else {
        (
            "dds/ati2",
            encode_classic_dds_header(
                texture.width,
                texture.height,
                mip_count,
                base_level_byte_size,
                false,
            ),
        )
    };
    let mut bytes = header;
    bytes.extend_from_slice(&payload);

    descriptor.format = format.to_string();
    texture.rgba.clear();
    texture.payload = TexturePayload::Container {
        format: format.to_string(),
        bytes,
        mip_count,
        array_layers: layer_count,
    };
    texture.descriptor = Some(descriptor);
    Ok(texture)
}

fn encode_bc5_payload(
    source: &[u8],
    width: u32,
    height: u32,
    mip_count: u32,
    layer_count: u32,
) -> Option<Vec<u8>> {
    let capacity = bc5_mip_chain_len(width, height, mip_count, layer_count)?;
    let mut payload = Vec::with_capacity(capacity);
    let mut mip_levels = Vec::with_capacity(mip_count as usize);
    let mut source_offset = 0_usize;
    for mip_level in 0..mip_count {
        let mip_width = mip_extent(width, mip_level);
        let mip_height = mip_extent(height, mip_level);
        let layer_len = rgba_level_len(mip_width, mip_height)?;
        mip_levels.push((mip_width, mip_height, layer_len, source_offset));
        source_offset = source_offset.checked_add(layer_len.checked_mul(layer_count as usize)?)?;
    }

    // DDS subresources are laid out as every mip of one array layer before the next layer.
    for layer_index in 0..layer_count as usize {
        for (mip_width, mip_height, layer_len, mip_offset) in &mip_levels {
            let layer_offset = mip_offset.checked_add(layer_index.checked_mul(*layer_len)?)?;
            let layer = source.get(layer_offset..layer_offset.checked_add(*layer_len)?)?;
            encode_bc5_layer(layer, *mip_width, *mip_height, &mut payload);
        }
    }
    Some(payload)
}

fn encode_bc5_layer(source: &[u8], width: u32, height: u32, output: &mut Vec<u8>) {
    for block_y in 0..height.div_ceil(BC5_BLOCK_EDGE) {
        for block_x in 0..width.div_ceil(BC5_BLOCK_EDGE) {
            let mut red = [0_u8; 16];
            let mut green = [0_u8; 16];
            let source_x = block_x * BC5_BLOCK_EDGE;
            let source_y = block_y * BC5_BLOCK_EDGE;
            if width >= BC5_BLOCK_EDGE
                && height >= BC5_BLOCK_EDGE
                && source_x <= width - BC5_BLOCK_EDGE
                && source_y <= height - BC5_BLOCK_EDGE
            {
                let row_stride = width as usize * 4;
                let block_offset = (source_y as usize * width as usize + source_x as usize) * 4;
                for local_y in 0..BC5_BLOCK_EDGE as usize {
                    let row_offset = block_offset + local_y * row_stride;
                    let target_offset = local_y * BC5_BLOCK_EDGE as usize;
                    red[target_offset] = source[row_offset];
                    red[target_offset + 1] = source[row_offset + 4];
                    red[target_offset + 2] = source[row_offset + 8];
                    red[target_offset + 3] = source[row_offset + 12];
                    green[target_offset] = source[row_offset + 1];
                    green[target_offset + 1] = source[row_offset + 5];
                    green[target_offset + 2] = source[row_offset + 9];
                    green[target_offset + 3] = source[row_offset + 13];
                }
            } else {
                for local_y in 0..BC5_BLOCK_EDGE {
                    for local_x in 0..BC5_BLOCK_EDGE {
                        let source_x = (source_x + local_x).min(width - 1);
                        let source_y = (source_y + local_y).min(height - 1);
                        let source_offset = ((source_y * width + source_x) as usize) * 4;
                        let index = (local_y * BC5_BLOCK_EDGE + local_x) as usize;
                        red[index] = source[source_offset];
                        green[index] = source[source_offset + 1];
                    }
                }
            }
            output.extend_from_slice(&encode_bc4_block(&red));
            output.extend_from_slice(&encode_bc4_block(&green));
        }
    }
}

fn encode_bc4_block(values: &[u8; 16]) -> [u8; 8] {
    let mut endpoint_high = values[0];
    let mut endpoint_low = values[0];
    for value in &values[1..] {
        endpoint_high = endpoint_high.max(*value);
        endpoint_low = endpoint_low.min(*value);
    }
    let palette = bc4_palette(endpoint_high, endpoint_low);
    let mut indices = 0_u64;
    for (index, value) in values.iter().enumerate() {
        let mut palette_index = 0;
        let mut nearest_distance = u16::MAX;
        for (candidate_index, candidate) in palette.iter().enumerate() {
            let distance = u16::from(*candidate).abs_diff(u16::from(*value));
            if distance < nearest_distance {
                palette_index = candidate_index;
                nearest_distance = distance;
                if distance == 0 {
                    break;
                }
            }
        }
        indices |= (palette_index as u64) << (index * 3);
    }
    let mut block = [0_u8; 8];
    block[0] = endpoint_high;
    block[1] = endpoint_low;
    block[2..8].copy_from_slice(&indices.to_le_bytes()[..6]);
    block
}

fn bc4_palette(endpoint_high: u8, endpoint_low: u8) -> [u8; 8] {
    if endpoint_high > endpoint_low {
        [
            endpoint_high,
            endpoint_low,
            ((6 * u16::from(endpoint_high) + u16::from(endpoint_low)) / 7) as u8,
            ((5 * u16::from(endpoint_high) + 2 * u16::from(endpoint_low)) / 7) as u8,
            ((4 * u16::from(endpoint_high) + 3 * u16::from(endpoint_low)) / 7) as u8,
            ((3 * u16::from(endpoint_high) + 4 * u16::from(endpoint_low)) / 7) as u8,
            ((2 * u16::from(endpoint_high) + 5 * u16::from(endpoint_low)) / 7) as u8,
            ((u16::from(endpoint_high) + 6 * u16::from(endpoint_low)) / 7) as u8,
        ]
    } else {
        [
            endpoint_high,
            endpoint_low,
            ((4 * u16::from(endpoint_high) + u16::from(endpoint_low)) / 5) as u8,
            ((3 * u16::from(endpoint_high) + 2 * u16::from(endpoint_low)) / 5) as u8,
            ((2 * u16::from(endpoint_high) + 3 * u16::from(endpoint_low)) / 5) as u8,
            ((u16::from(endpoint_high) + 4 * u16::from(endpoint_low)) / 5) as u8,
            0,
            255,
        ]
    }
}

fn encode_classic_dds_header(
    width: u32,
    height: u32,
    mip_count: u32,
    base_level_byte_size: u32,
    cube: bool,
) -> Vec<u8> {
    let mut header = encode_dds_header(
        width,
        height,
        mip_count,
        base_level_byte_size,
        cube,
        DDS_FOURCC_ATI2,
        DDS_CLASSIC_HEADER_LEN,
    );
    if cube {
        write_u32_le(&mut header, 112, DDS_CAPS2_CUBEMAP_ALL_FACES);
    }
    header
}

fn encode_dx10_array_dds_header(
    width: u32,
    height: u32,
    mip_count: u32,
    base_level_byte_size: u32,
    array_layer_count: u32,
) -> Vec<u8> {
    let mut header = encode_dds_header(
        width,
        height,
        mip_count,
        base_level_byte_size,
        array_layer_count > 1,
        DDS_FOURCC_DX10,
        DDS_DX10_HEADER_LEN,
    );
    write_u32_le(&mut header, 128, DXGI_FORMAT_BC5_UNORM);
    write_u32_le(&mut header, 132, DDS_RESOURCE_DIMENSION_TEXTURE2D);
    write_u32_le(&mut header, 136, 0);
    write_u32_le(&mut header, 140, array_layer_count);
    write_u32_le(&mut header, 144, 0);
    header
}

fn encode_dds_header(
    width: u32,
    height: u32,
    mip_count: u32,
    base_level_byte_size: u32,
    complex: bool,
    fourcc: u32,
    header_len: usize,
) -> Vec<u8> {
    let mut header = vec![0_u8; header_len];
    header[..4].copy_from_slice(b"DDS ");
    write_u32_le(&mut header, 4, 124);
    write_u32_le(
        &mut header,
        8,
        if mip_count > 1 {
            DDS_FLAGS_WITH_MIPS
        } else {
            DDS_FLAGS
        },
    );
    write_u32_le(&mut header, 12, height);
    write_u32_le(&mut header, 16, width);
    write_u32_le(&mut header, 20, base_level_byte_size);
    write_u32_le(&mut header, 28, mip_count);
    write_u32_le(&mut header, 76, 32);
    write_u32_le(&mut header, 80, DDS_PIXEL_FORMAT_FOURCC);
    write_u32_le(&mut header, 84, fourcc);
    let caps = DDS_CAPS_TEXTURE
        | if mip_count > 1 || complex {
            DDS_CAPS_COMPLEX
        } else {
            0
        }
        | if mip_count > 1 { DDS_CAPS_MIPMAP } else { 0 };
    write_u32_le(&mut header, 108, caps);
    header
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn rgba_mip_chain_len(width: u32, height: u32, mip_count: u32, layer_count: u32) -> Option<usize> {
    (0..mip_count).try_fold(0_usize, |total, level| {
        let level_len = rgba_level_len(mip_extent(width, level), mip_extent(height, level))?;
        total.checked_add(level_len.checked_mul(layer_count as usize)?)
    })
}

fn bc5_mip_chain_len(width: u32, height: u32, mip_count: u32, layer_count: u32) -> Option<usize> {
    (0..mip_count).try_fold(0_usize, |total, level| {
        let level_len = bc5_level_len(mip_extent(width, level), mip_extent(height, level))?;
        total.checked_add(level_len.checked_mul(layer_count as usize)?)
    })
}

fn rgba_level_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)
}

fn bc5_level_len(width: u32, height: u32) -> Option<usize> {
    (width.div_ceil(BC5_BLOCK_EDGE) as usize)
        .checked_mul(height.div_ceil(BC5_BLOCK_EDGE) as usize)?
        .checked_mul(BC5_BLOCK_BYTES)
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    if level >= u32::BITS {
        1
    } else {
        (value >> level).max(1)
    }
}

#[cfg(test)]
#[path = "tests/bc5.rs"]
mod tests;
