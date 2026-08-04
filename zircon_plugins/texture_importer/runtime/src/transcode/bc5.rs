use zircon_runtime::asset::{AssetImportError, TextureAsset, TexturePayload};
use zircon_runtime::core::framework::render::{
    RenderImageDimension, TextureCompressionTarget, TextureUsageHint,
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
            for local_y in 0..BC5_BLOCK_EDGE {
                for local_x in 0..BC5_BLOCK_EDGE {
                    let source_x = (block_x * BC5_BLOCK_EDGE + local_x).min(width - 1);
                    let source_y = (block_y * BC5_BLOCK_EDGE + local_y).min(height - 1);
                    let source_offset = ((source_y * width + source_x) as usize) * 4;
                    let index = (local_y * BC5_BLOCK_EDGE + local_x) as usize;
                    red[index] = source[source_offset];
                    green[index] = source[source_offset + 1];
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
mod tests {
    use super::*;
    use zircon_runtime::asset::{AssetUri, TextureUploadSupport};
    use zircon_runtime::core::framework::render::{
        TextureMetadata, TextureMipPolicy, TextureNormalConvention,
    };

    #[test]
    fn normal_bc5_transcode_emits_upload_ready_ati2_payload() {
        let mut texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/normal.png").expect("valid texture uri"),
            4,
            4,
            vec![128, 255, 0, 255].repeat(16),
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            compression: TextureCompressionTarget::Bc5,
            normal_convention: TextureNormalConvention::TangentSpaceDx,
            mip_policy: TextureMipPolicy::FromSource,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);

        let texture = transcode_normal_bc5(texture).expect("bc5 transcode succeeds");

        assert!(texture.rgba.is_empty());
        assert_eq!(texture.texture_descriptor().format, "dds/ati2");
        assert!(
            texture
                .upload_readiness(TextureUploadSupport::all_compressed())
                .is_ready()
        );
        let TexturePayload::Container {
            bytes, mip_count, ..
        } = texture.payload
        else {
            panic!("bc5 transcode must produce a container payload");
        };
        assert_eq!(mip_count, 1);
        assert_eq!(&bytes[..4], b"DDS ");
        assert_eq!(&bytes[84..88], b"ATI2");
        assert_eq!(bytes.len(), DDS_CLASSIC_HEADER_LEN + BC5_BLOCK_BYTES);
    }

    #[test]
    fn normal_bc5_mip_chain_transcode_emits_upload_ready_dds_payload() {
        let mut texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/normal-mips.png").expect("valid texture uri"),
            4,
            4,
            vec![128, 255, 0, 255].repeat(4 * 4 + 2 * 2 + 1),
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.mip_count = 3;
        descriptor.metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            compression: TextureCompressionTarget::Bc5,
            normal_convention: TextureNormalConvention::TangentSpaceDx,
            mip_policy: TextureMipPolicy::FromSource,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);

        let texture = transcode_normal_bc5(texture).expect("bc5 mip transcode succeeds");

        assert!(
            texture
                .upload_readiness(TextureUploadSupport::all_compressed())
                .is_ready()
        );
        let TexturePayload::Container {
            bytes, mip_count, ..
        } = texture.payload
        else {
            panic!("bc5 mip transcode must produce a container payload");
        };
        assert_eq!(mip_count, 3);
        assert_eq!(bytes.len(), DDS_CLASSIC_HEADER_LEN + 3 * BC5_BLOCK_BYTES);
    }

    #[test]
    fn bc5_palette_uses_full_range_when_all_texels_match() {
        let block = encode_bc4_block(&[64; 16]);

        assert_eq!(block[0], 64);
        assert_eq!(block[1], 64);
        assert!(block[2..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn existing_normal_container_is_preserved_without_a_transcoder() {
        let mut texture = TextureAsset::new_container(
            AssetUri::parse("res://textures/existing-normal.dds").expect("valid texture uri"),
            4,
            4,
            "dds/dxt5",
            vec![0; DDS_CLASSIC_HEADER_LEN],
            1,
            1,
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            compression: TextureCompressionTarget::Bc5,
            normal_convention: TextureNormalConvention::TangentSpaceDx,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);

        let texture = transcode_normal_bc5(texture).expect("container is retained");

        assert_eq!(texture.texture_descriptor().format, "dds/dxt5");
        assert!(matches!(texture.payload, TexturePayload::Container { .. }));
    }

    #[test]
    fn cubemap_dds_header_marks_complex_and_all_faces() {
        let header = encode_classic_dds_header(4, 4, 1, BC5_BLOCK_BYTES as u32, true);

        assert_ne!(
            u32::from_le_bytes(header[108..112].try_into().unwrap()) & DDS_CAPS_COMPLEX,
            0
        );
        assert_eq!(
            u32::from_le_bytes(header[112..116].try_into().unwrap()),
            DDS_CAPS2_CUBEMAP_ALL_FACES
        );
    }

    #[test]
    fn normal_bc5_array_transcode_emits_dx10_upload_payload() {
        let mut texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/normal-array.png").expect("valid texture uri"),
            4,
            4,
            vec![128, 255, 0, 255].repeat(32),
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.depth_or_array_layers = 2;
        descriptor.array_layer_count = 2;
        descriptor.metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            compression: TextureCompressionTarget::Bc5,
            normal_convention: TextureNormalConvention::TangentSpaceDx,
            mip_policy: TextureMipPolicy::FromSource,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);

        let texture = transcode_normal_bc5(texture).expect("bc5 array transcode succeeds");

        assert_eq!(texture.texture_descriptor().format, "dds/dxgi-83");
        assert!(
            texture
                .upload_readiness(TextureUploadSupport::all_compressed())
                .is_ready()
        );
        let TexturePayload::Container {
            bytes,
            mip_count,
            array_layers,
            ..
        } = texture.payload
        else {
            panic!("bc5 array transcode must produce a container payload");
        };
        assert_eq!(mip_count, 1);
        assert_eq!(array_layers, 2);
        assert_eq!(&bytes[84..88], b"DX10");
        assert_eq!(
            u32::from_le_bytes(bytes[128..132].try_into().unwrap()),
            DXGI_FORMAT_BC5_UNORM
        );
        assert_eq!(u32::from_le_bytes(bytes[140..144].try_into().unwrap()), 2);
        assert_eq!(bytes.len(), DDS_DX10_HEADER_LEN + 2 * BC5_BLOCK_BYTES);
    }
}
