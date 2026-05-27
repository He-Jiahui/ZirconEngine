use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderImageDimension;

use super::{TextureAsset, TexturePayload, RGBA8_UNORM_SRGB_FORMAT};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureUploadSupport {
    pub bc: bool,
    pub bc_sliced_3d: bool,
    pub etc2: bool,
    pub astc_ldr: bool,
    pub astc_sliced_3d: bool,
}

impl TextureUploadSupport {
    pub const fn uncompressed_only() -> Self {
        Self {
            bc: false,
            bc_sliced_3d: false,
            etc2: false,
            astc_ldr: false,
            astc_sliced_3d: false,
        }
    }

    pub const fn all_compressed() -> Self {
        Self {
            bc: true,
            bc_sliced_3d: true,
            etc2: true,
            astc_ldr: true,
            astc_sliced_3d: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureUploadCompressionFamily {
    Uncompressed,
    Bc,
    Etc2,
    Astc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureUploadPlan {
    pub format: String,
    pub compression: TextureUploadCompressionFamily,
    pub data_offset: usize,
    /// Declared byte length for bounded container levels such as KTX mip payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_length: Option<usize>,
    pub block_width: u32,
    pub block_height: u32,
    pub block_depth: u32,
    pub bytes_per_block: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TextureUploadReadiness {
    Ready { plan: TextureUploadPlan },
    Unsupported { reason: String },
}

impl TextureUploadReadiness {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match self {
            Self::Ready { .. } => None,
            Self::Unsupported { reason } => Some(reason.as_str()),
        }
    }
}

impl TextureAsset {
    pub fn upload_readiness(&self, support: TextureUploadSupport) -> TextureUploadReadiness {
        match &self.payload {
            TexturePayload::Rgba8 => rgba8_upload_readiness(self),
            TexturePayload::Container { format, bytes, .. } => {
                container_upload_readiness(self, format, bytes, support)
            }
        }
    }
}

fn rgba8_upload_readiness(texture: &TextureAsset) -> TextureUploadReadiness {
    if let Some(reason) = unsupported_rgba8_shape_reason(texture) {
        return unsupported(reason);
    }
    let Some(expected_len) = rgba8_len(texture.width, texture.height) else {
        return unsupported("rgba8 texture extent is too large to upload");
    };
    if texture.rgba.len() != expected_len {
        return unsupported(format!(
            "rgba8 texture payload length {} does not match expected {}",
            texture.rgba.len(),
            expected_len
        ));
    }
    ready(TextureUploadPlan {
        format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
        compression: TextureUploadCompressionFamily::Uncompressed,
        data_offset: 0,
        data_length: Some(texture.rgba.len()),
        block_width: 1,
        block_height: 1,
        block_depth: 1,
        bytes_per_block: 4,
    })
}

fn unsupported_rgba8_shape_reason(texture: &TextureAsset) -> Option<String> {
    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension == RenderImageDimension::D1 {
        return Some("rgba8 texture 1d upload is not implemented".to_string());
    }
    if descriptor.dimension == RenderImageDimension::D3 {
        return Some("rgba8 texture 3d upload is not implemented".to_string());
    }
    if descriptor.mip_count > 1 {
        return Some("rgba8 texture mip-chain upload is not implemented".to_string());
    }
    if descriptor.dimension == RenderImageDimension::D2
        && (descriptor.array_layer_count > 1 || descriptor.depth_or_array_layers > 1)
    {
        return Some("rgba8 texture array/cubemap upload is not implemented".to_string());
    }
    None
}
fn container_upload_readiness(
    texture: &TextureAsset,
    format: &str,
    bytes: &[u8],
    support: TextureUploadSupport,
) -> TextureUploadReadiness {
    if let Some(plan) = dds_upload_plan(format) {
        return compressed_plan_readiness(texture, bytes, plan, support);
    }
    if let Some(plan) = astc_upload_plan(format) {
        return compressed_plan_readiness(texture, bytes, plan, support);
    }
    if format.starts_with("ktx2/") {
        if let Some(supercompression) = ktx2_supercompression(format) {
            if supercompression != 0 {
                return unsupported(format!(
                    "ktx2 supercompression {supercompression} requires a transcoding backend"
                ));
            }
        }
        if let Some(plan) = ktx2_upload_plan(format, bytes) {
            return compressed_plan_readiness(texture, bytes, plan, support);
        }
        return unsupported("ktx2 texture format or level index is not upload-ready");
    }
    if format.starts_with("ktx/") {
        if let Some(plan) = ktx_upload_plan(format, bytes) {
            return compressed_plan_readiness(texture, bytes, plan, support);
        }
        return unsupported("ktx texture format or level payload is not upload-ready");
    }
    unsupported(format!(
        "texture container format {format} is not upload-ready"
    ))
}

fn compressed_plan_readiness(
    texture: &TextureAsset,
    bytes: &[u8],
    plan: TextureUploadPlan,
    support: TextureUploadSupport,
) -> TextureUploadReadiness {
    if let Some(reason) = unsupported_feature_reason(texture, &plan, support) {
        return unsupported(reason);
    }
    if let Some(reason) = unsupported_container_shape_reason(texture) {
        return unsupported(reason);
    }
    if bytes.len() <= plan.data_offset {
        return unsupported(format!(
            "container texture payload format {} has no image data after {} byte header",
            plan.format, plan.data_offset
        ));
    }
    let Some(required_bytes) = compressed_mip0_required_len(texture, &plan) else {
        return unsupported(format!(
            "container texture payload format {} upload size overflows",
            plan.format
        ));
    };
    let available_bytes = bytes.len() - plan.data_offset;
    if let Some(data_length) = plan.data_length {
        if available_bytes < data_length {
            return unsupported(format!(
                "container texture payload format {} declares {} image bytes but only {} are available",
                plan.format, data_length, available_bytes
            ));
        }
        if data_length < required_bytes {
            return unsupported(format!(
                "container texture payload format {} declares {} image bytes but needs at least {}",
                plan.format, data_length, required_bytes
            ));
        }
    }
    if available_bytes < required_bytes {
        return unsupported(format!(
            "container texture payload format {} has {} image bytes but needs at least {}",
            plan.format, available_bytes, required_bytes
        ));
    }
    ready(plan)
}

fn compressed_mip0_required_len(texture: &TextureAsset, plan: &TextureUploadPlan) -> Option<usize> {
    let descriptor = texture.render_image_descriptor();
    let block_columns = div_ceil(texture.width.max(1), plan.block_width.max(1));
    let block_rows = div_ceil(texture.height.max(1), plan.block_height.max(1));
    let layer_count = descriptor.depth_or_array_layers.max(1);
    let bytes = block_columns
        .checked_mul(block_rows)?
        .checked_mul(layer_count)?
        .checked_mul(plan.bytes_per_block)?;
    usize::try_from(bytes).ok()
}

fn unsupported_container_shape_reason(texture: &TextureAsset) -> Option<String> {
    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension == RenderImageDimension::D1 {
        return Some("compressed texture 1d upload is not implemented".to_string());
    }
    if descriptor.mip_count > 1 {
        return Some("compressed texture mip-chain upload is not implemented".to_string());
    }
    if descriptor.dimension == RenderImageDimension::D2
        && (descriptor.array_layer_count > 1 || descriptor.depth_or_array_layers > 1)
    {
        return Some("compressed texture array/cubemap upload is not implemented".to_string());
    }
    None
}

fn unsupported_feature_reason(
    texture: &TextureAsset,
    plan: &TextureUploadPlan,
    support: TextureUploadSupport,
) -> Option<String> {
    match plan.compression {
        TextureUploadCompressionFamily::Uncompressed => None,
        TextureUploadCompressionFamily::Bc if !support.bc => {
            Some("gpu device does not support BC compressed textures".to_string())
        }
        TextureUploadCompressionFamily::Bc
            if texture.render_image_descriptor().dimension == RenderImageDimension::D3
                && !support.bc_sliced_3d =>
        {
            Some("gpu device does not support BC sliced 3d textures".to_string())
        }
        TextureUploadCompressionFamily::Bc => None,
        TextureUploadCompressionFamily::Etc2 if !support.etc2 => {
            Some("gpu device does not support ETC2 compressed textures".to_string())
        }
        TextureUploadCompressionFamily::Etc2
            if texture.render_image_descriptor().dimension == RenderImageDimension::D3 =>
        {
            Some("compressed texture ETC2 3d upload is not implemented".to_string())
        }
        TextureUploadCompressionFamily::Etc2 => None,
        TextureUploadCompressionFamily::Astc if !support.astc_ldr => {
            Some("gpu device does not support ASTC compressed textures".to_string())
        }
        TextureUploadCompressionFamily::Astc
            if (plan.block_depth > 1
                || texture.render_image_descriptor().dimension == RenderImageDimension::D3)
                && !support.astc_sliced_3d =>
        {
            Some("gpu device does not support ASTC sliced 3d textures".to_string())
        }
        TextureUploadCompressionFamily::Astc if plan.block_depth > 1 => {
            Some("astc 3d block payload upload is not implemented".to_string())
        }
        TextureUploadCompressionFamily::Astc => None,
    }
}

fn dds_upload_plan(format: &str) -> Option<TextureUploadPlan> {
    let (normalized, data_offset, bytes_per_block) =
        match format.trim().to_ascii_lowercase().as_str() {
            "dds/dxt1" => ("dds/dxt1".to_string(), 128, 8),
            "dds/dxt3" => ("dds/dxt3".to_string(), 128, 16),
            "dds/dxt5" => ("dds/dxt5".to_string(), 128, 16),
            "dds/ati1" => ("dds/ati1".to_string(), 128, 8),
            "dds/bc4u" => ("dds/bc4u".to_string(), 128, 8),
            "dds/bc4s" => ("dds/bc4s".to_string(), 128, 8),
            "dds/ati2" => ("dds/ati2".to_string(), 128, 16),
            "dds/bc5u" => ("dds/bc5u".to_string(), 128, 16),
            "dds/bc5s" => ("dds/bc5s".to_string(), 128, 16),
            value if value.starts_with("dds/dxgi-") => {
                let dxgi = value.trim_start_matches("dds/dxgi-").parse::<u32>().ok()?;
                let bytes_per_block = match dxgi {
                    71 | 72 | 80 | 81 => 8,
                    74 | 75 | 77 | 78 | 83 | 84 | 95 | 96 | 98 | 99 => 16,
                    _ => return None,
                };
                (value.to_string(), 148, bytes_per_block)
            }
            _ => return None,
        };
    Some(TextureUploadPlan {
        format: normalized,
        compression: TextureUploadCompressionFamily::Bc,
        data_offset,
        data_length: None,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
    })
}

fn ktx_upload_plan(format: &str, bytes: &[u8]) -> Option<TextureUploadPlan> {
    let normalized = format.trim().to_ascii_lowercase();
    if bytes.get(..KTX1_IDENTIFIER.len())? != KTX1_IDENTIFIER {
        return None;
    }
    if read_u32_le(bytes, 12)? != KTX_LITTLE_ENDIAN {
        return None;
    }
    let gl_internal_format = parse_ktx_gl_internal_format(&normalized)?;
    if read_u32_le(bytes, 28)? != gl_internal_format {
        return None;
    }
    let layout = ktx_gl_compressed_layout(gl_internal_format)?;
    let key_value_data_len = usize::try_from(read_u32_le(bytes, 60)?).ok()?;
    let image_size_offset = 64_usize.checked_add(key_value_data_len)?;
    let data_length = usize::try_from(read_u32_le(bytes, image_size_offset)?).ok()?;
    let data_offset = image_size_offset.checked_add(4)?;

    Some(TextureUploadPlan {
        format: normalized,
        compression: layout.compression,
        data_offset,
        data_length: Some(data_length),
        block_width: layout.block_width,
        block_height: layout.block_height,
        block_depth: layout.block_depth,
        bytes_per_block: layout.bytes_per_block,
    })
}

fn ktx2_upload_plan(format: &str, bytes: &[u8]) -> Option<TextureUploadPlan> {
    let normalized = format.trim().to_ascii_lowercase();
    if bytes.get(..KTX2_IDENTIFIER.len())? != KTX2_IDENTIFIER {
        return None;
    }
    let vk_format = parse_ktx2_vk_format(&normalized)?;
    if read_u32_le(bytes, 12)? != vk_format {
        return None;
    }
    let supercompression = ktx2_supercompression(&normalized)?;
    if read_u32_le(bytes, 44)? != supercompression {
        return None;
    }
    let layout = ktx2_vk_compressed_layout(vk_format)?;
    let level_data_offset = usize::try_from(read_u64_le(bytes, KTX2_LEVEL_INDEX_OFFSET)?).ok()?;
    let level_data_len = usize::try_from(read_u64_le(bytes, KTX2_LEVEL_INDEX_OFFSET + 8)?).ok()?;
    let level_uncompressed_len =
        usize::try_from(read_u64_le(bytes, KTX2_LEVEL_INDEX_OFFSET + 16)?).ok()?;
    if level_data_len == 0 {
        return None;
    }
    let level_count = usize::try_from(read_u32_le(bytes, 40)?.max(1)).ok()?;
    let level_index_end = KTX2_LEVEL_INDEX_OFFSET
        .checked_add(level_count.checked_mul(KTX2_LEVEL_INDEX_ENTRY_SIZE)?)?;
    if bytes.len() < level_index_end || level_data_offset < level_index_end {
        return None;
    }
    let declared_level_is_short = bytes.len().saturating_sub(level_data_offset) < level_data_len;
    if supercompression == 0 && level_uncompressed_len != level_data_len && !declared_level_is_short
    {
        return None;
    }

    Some(TextureUploadPlan {
        format: normalized,
        compression: layout.compression,
        data_offset: level_data_offset,
        data_length: Some(level_data_len),
        block_width: layout.block_width,
        block_height: layout.block_height,
        block_depth: layout.block_depth,
        bytes_per_block: layout.bytes_per_block,
    })
}

#[derive(Clone, Copy)]
struct CompressedFormatLayout {
    compression: TextureUploadCompressionFamily,
    block_width: u32,
    block_height: u32,
    block_depth: u32,
    bytes_per_block: u32,
}

fn bc_layout(bytes_per_block: u32) -> CompressedFormatLayout {
    CompressedFormatLayout {
        compression: TextureUploadCompressionFamily::Bc,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
    }
}

fn etc2_layout(bytes_per_block: u32) -> CompressedFormatLayout {
    CompressedFormatLayout {
        compression: TextureUploadCompressionFamily::Etc2,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
    }
}

fn astc_layout(block_width: u32, block_height: u32) -> CompressedFormatLayout {
    CompressedFormatLayout {
        compression: TextureUploadCompressionFamily::Astc,
        block_width,
        block_height,
        block_depth: 1,
        bytes_per_block: 16,
    }
}

fn ktx_gl_compressed_layout(gl_internal_format: u32) -> Option<CompressedFormatLayout> {
    match gl_internal_format {
        // S3TC / BC1, BC2, BC3 including sRGB variants.
        0x83f0 | 0x83f1 | 0x8c4c | 0x8c4d => Some(bc_layout(8)),
        0x83f2 | 0x83f3 | 0x8c4e | 0x8c4f => Some(bc_layout(16)),
        // RGTC / BPTC forms of BC4, BC5, BC6H, and BC7.
        0x8dbb | 0x8dbc => Some(bc_layout(8)),
        0x8dbd | 0x8dbe | 0x8e8c | 0x8e8d | 0x8e8e | 0x8e8f => Some(bc_layout(16)),
        // ETC2 / EAC formats accepted by wgpu's ETC2 family.
        0x9274 | 0x9275 | 0x9276 | 0x9277 | 0x9270 | 0x9271 => Some(etc2_layout(8)),
        0x9278 | 0x9279 | 0x9272 | 0x9273 => Some(etc2_layout(16)),
        0x93b0..=0x93bd | 0x93d0..=0x93dd => {
            let (block_width, block_height) = ktx_gl_astc_block(gl_internal_format)?;
            Some(astc_layout(block_width, block_height))
        }
        _ => None,
    }
}

fn ktx_gl_astc_block(gl_internal_format: u32) -> Option<(u32, u32)> {
    let index = if (0x93b0..=0x93bd).contains(&gl_internal_format) {
        gl_internal_format - 0x93b0
    } else if (0x93d0..=0x93dd).contains(&gl_internal_format) {
        gl_internal_format - 0x93d0
    } else {
        return None;
    };
    astc_2d_block_by_index(index)
}

fn ktx2_vk_compressed_layout(vk_format: u32) -> Option<CompressedFormatLayout> {
    match vk_format {
        // VK_FORMAT_BC1_*_BLOCK
        131..=134 => Some(bc_layout(8)),
        // VK_FORMAT_BC2/BC3_*_BLOCK
        135..=138 => Some(bc_layout(16)),
        // VK_FORMAT_BC4_*_BLOCK
        139 | 140 => Some(bc_layout(8)),
        // VK_FORMAT_BC5/BC6H/BC7_*_BLOCK
        141..=146 => Some(bc_layout(16)),
        // VK_FORMAT_ETC2_* and VK_FORMAT_EAC_*_BLOCK
        147..=150 | 153 | 154 => Some(etc2_layout(8)),
        151 | 152 | 155 | 156 => Some(etc2_layout(16)),
        157..=184 => {
            let (block_width, block_height) = ktx2_astc_block(vk_format)?;
            Some(astc_layout(block_width, block_height))
        }
        _ => None,
    }
}

fn ktx2_astc_block(vk_format: u32) -> Option<(u32, u32)> {
    if !(157..=184).contains(&vk_format) {
        return None;
    }
    astc_2d_block_by_index((vk_format - 157) / 2)
}

fn astc_2d_block_by_index(index: u32) -> Option<(u32, u32)> {
    Some(match index {
        0 => (4, 4),
        1 => (5, 4),
        2 => (5, 5),
        3 => (6, 5),
        4 => (6, 6),
        5 => (8, 5),
        6 => (8, 6),
        7 => (8, 8),
        8 => (10, 5),
        9 => (10, 6),
        10 => (10, 8),
        11 => (10, 10),
        12 => (12, 10),
        13 => (12, 12),
        _ => return None,
    })
}

fn astc_upload_plan(format: &str) -> Option<TextureUploadPlan> {
    let value = format.trim().to_ascii_lowercase();
    let dimensions = value.strip_prefix("astc/")?;
    let mut parts = dimensions.split('x');
    let block_width = parts.next()?.parse::<u32>().ok()?;
    let block_height = parts.next()?.parse::<u32>().ok()?;
    let block_depth = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some()
        || block_width == 0
        || block_height == 0
        || block_depth == 0
        || !is_supported_astc_block(block_width, block_height, block_depth)
    {
        return None;
    }
    Some(TextureUploadPlan {
        format: value,
        compression: TextureUploadCompressionFamily::Astc,
        data_offset: 16,
        data_length: None,
        block_width,
        block_height,
        block_depth,
        bytes_per_block: 16,
    })
}

fn is_supported_astc_block(width: u32, height: u32, depth: u32) -> bool {
    if depth == 1 {
        return matches!(
            (width, height),
            (4, 4)
                | (5, 4)
                | (5, 5)
                | (6, 5)
                | (6, 6)
                | (8, 5)
                | (8, 6)
                | (8, 8)
                | (10, 5)
                | (10, 6)
                | (10, 8)
                | (10, 10)
                | (12, 10)
                | (12, 12)
        );
    }

    // ASTC 3D block dimensions are codec-defined, not runtime policy.
    matches!(
        (width, height, depth),
        (3, 3, 3)
            | (4, 3, 3)
            | (4, 4, 3)
            | (4, 4, 4)
            | (5, 4, 4)
            | (5, 5, 4)
            | (5, 5, 5)
            | (6, 5, 5)
            | (6, 6, 5)
            | (6, 6, 6)
    )
}

fn ktx2_supercompression(format: &str) -> Option<u32> {
    format
        .split('/')
        .find_map(|part| part.strip_prefix("supercompression-"))
        .and_then(|value| value.parse::<u32>().ok())
}

fn parse_ktx_gl_internal_format(format: &str) -> Option<u32> {
    let value = format.strip_prefix("ktx/gl-internal-0x")?;
    u32::from_str_radix(value, 16).ok()
}

fn parse_ktx2_vk_format(format: &str) -> Option<u32> {
    format
        .split('/')
        .find_map(|part| part.strip_prefix("vk-"))
        .and_then(|value| value.parse::<u32>().ok())
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Option<u64> {
    let slice = bytes.get(offset..offset.checked_add(8)?)?;
    Some(u64::from_le_bytes(slice.try_into().ok()?))
}

fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor.max(1)
}

const KTX_LITTLE_ENDIAN: u32 = 0x0403_0201;
const KTX1_IDENTIFIER: &[u8] = b"\xABKTX 11\xBB\r\n\x1A\n";
const KTX2_IDENTIFIER: &[u8] = b"\xABKTX 20\xBB\r\n\x1A\n";
const KTX2_LEVEL_INDEX_OFFSET: usize = 80;
const KTX2_LEVEL_INDEX_ENTRY_SIZE: usize = 24;

fn rgba8_len(width: u32, height: u32) -> Option<usize> {
    width
        .checked_mul(height)?
        .checked_mul(4)
        .and_then(|bytes| usize::try_from(bytes).ok())
}

fn ready(plan: TextureUploadPlan) -> TextureUploadReadiness {
    TextureUploadReadiness::Ready { plan }
}

fn unsupported(reason: impl Into<String>) -> TextureUploadReadiness {
    TextureUploadReadiness::Unsupported {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetUri;
    use crate::asset::TextureAssetDescriptor;

    #[test]
    fn ktx2_upload_plan_rejects_level_payload_inside_level_index() {
        let mut bytes = ktx2_bc1_level_bytes();
        write_u64_le(&mut bytes, KTX2_LEVEL_INDEX_OFFSET, 88);
        let texture = TextureAsset::new_container(
            AssetUri::parse("res://textures/overlapping-index.ktx2").unwrap(),
            4,
            4,
            "ktx2/vk-133/supercompression-0",
            bytes,
            1,
            1,
        );

        assert_eq!(
            texture
                .upload_readiness(TextureUploadSupport {
                    bc: true,
                    ..TextureUploadSupport::uncompressed_only()
                })
                .unsupported_reason(),
            Some("ktx2 texture format or level index is not upload-ready")
        );
    }

    #[test]
    fn rgba8_upload_readiness_rejects_layered_shapes_before_byte_length_check() {
        let mut array_descriptor = TextureAssetDescriptor::rgba8_srgb();
        array_descriptor.depth_or_array_layers = 2;
        array_descriptor.array_layer_count = 2;
        let array_texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/stacked-array.png").unwrap(),
            2,
            2,
            vec![0_u8; 2 * 2 * 2 * 4],
        )
        .with_descriptor(array_descriptor);
        assert_eq!(
            array_texture
                .upload_readiness(TextureUploadSupport::uncompressed_only())
                .unsupported_reason(),
            Some("rgba8 texture array/cubemap upload is not implemented")
        );

        let mut volume_descriptor = TextureAssetDescriptor::rgba8_srgb();
        volume_descriptor.dimension = RenderImageDimension::D3;
        volume_descriptor.depth_or_array_layers = 4;
        volume_descriptor.array_layer_count = 1;
        let volume_texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/volume.png").unwrap(),
            2,
            2,
            vec![0_u8; 2 * 2 * 4 * 4],
        )
        .with_descriptor(volume_descriptor);
        assert_eq!(
            volume_texture
                .upload_readiness(TextureUploadSupport::uncompressed_only())
                .unsupported_reason(),
            Some("rgba8 texture 3d upload is not implemented")
        );

        let mut mip_descriptor = TextureAssetDescriptor::rgba8_srgb();
        mip_descriptor.mip_count = 2;
        let mip_texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/mips.png").unwrap(),
            2,
            2,
            vec![0_u8; 2 * 2 * 4],
        )
        .with_descriptor(mip_descriptor);
        assert_eq!(
            mip_texture
                .upload_readiness(TextureUploadSupport::uncompressed_only())
                .unsupported_reason(),
            Some("rgba8 texture mip-chain upload is not implemented")
        );
    }
    fn ktx2_bc1_level_bytes() -> Vec<u8> {
        let mut bytes = vec![0_u8; 104];
        bytes[0..12].copy_from_slice(KTX2_IDENTIFIER);
        write_u32_le(&mut bytes, 12, 133);
        write_u32_le(&mut bytes, 16, 1);
        write_u32_le(&mut bytes, 20, 4);
        write_u32_le(&mut bytes, 24, 4);
        write_u32_le(&mut bytes, 40, 1);
        write_u32_le(&mut bytes, 44, 0);
        write_u64_le(&mut bytes, 80, 104);
        write_u64_le(&mut bytes, 88, 8);
        write_u64_le(&mut bytes, 96, 8);
        bytes.extend_from_slice(&[1_u8; 8]);
        bytes
    }

    fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
}
