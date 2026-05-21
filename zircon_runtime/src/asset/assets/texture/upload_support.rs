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
        block_width: 1,
        block_height: 1,
        block_depth: 1,
        bytes_per_block: 4,
    })
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
        return unsupported("ktx2 level payload extraction is not implemented");
    }
    if format.starts_with("ktx/") {
        return unsupported("ktx level payload extraction is not implemented");
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
    ready(plan)
}

fn unsupported_container_shape_reason(texture: &TextureAsset) -> Option<String> {
    let descriptor = texture.render_image_descriptor();
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
            value if value.starts_with("dds/dxgi-") => {
                let dxgi = value.trim_start_matches("dds/dxgi-").parse::<u32>().ok()?;
                let bytes_per_block = match dxgi {
                    71 | 72 => 8,
                    74 | 75 | 77 | 78 | 98 | 99 => 16,
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
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
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
        || !is_supported_astc_2d_block(block_width, block_height)
    {
        return None;
    }
    Some(TextureUploadPlan {
        format: value,
        compression: TextureUploadCompressionFamily::Astc,
        data_offset: 16,
        block_width,
        block_height,
        block_depth,
        bytes_per_block: 16,
    })
}

fn is_supported_astc_2d_block(width: u32, height: u32) -> bool {
    matches!(
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
    )
}

fn ktx2_supercompression(format: &str) -> Option<u32> {
    format
        .split('/')
        .find_map(|part| part.strip_prefix("supercompression-"))
        .and_then(|value| value.parse::<u32>().ok())
}

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
