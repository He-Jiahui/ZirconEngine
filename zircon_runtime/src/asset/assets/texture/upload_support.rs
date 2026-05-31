mod astc;
mod bytes;
mod compressed;
mod dds;
mod ktx;
mod layout;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderImageDimension;

use self::astc::astc_upload_plan;
use self::compressed::compressed_plan_readiness;
use self::dds::dds_upload_plan;
use self::ktx::{ktx2_supercompression, ktx2_upload_plan, ktx_upload_plan};
use super::{TextureAsset, TexturePayload, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT};

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
    let descriptor = texture.render_image_descriptor();
    let Some(format) = rgba8_upload_format(&descriptor.format) else {
        return unsupported(format!(
            "rgba8 texture descriptor format {} requires conversion before upload",
            descriptor.format
        ));
    };
    if texture.rgba.len() != expected_len {
        return unsupported(format!(
            "rgba8 texture payload length {} does not match expected {}",
            texture.rgba.len(),
            expected_len
        ));
    }
    ready(TextureUploadPlan {
        format: format.to_string(),
        compression: TextureUploadCompressionFamily::Uncompressed,
        data_offset: 0,
        data_length: Some(texture.rgba.len()),
        block_width: 1,
        block_height: 1,
        block_depth: 1,
        bytes_per_block: 4,
    })
}

fn rgba8_upload_format(format: &str) -> Option<&'static str> {
    if format.trim().eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) {
        Some(RGBA8_UNORM_FORMAT)
    } else if format.trim().eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) {
        Some(RGBA8_UNORM_SRGB_FORMAT)
    } else {
        None
    }
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
    if let Some(plan) = dds_upload_plan(texture, format, bytes) {
        return compressed_plan_readiness(texture, bytes, plan, support);
    }
    if let Some(plan) = astc_upload_plan(texture, format, bytes) {
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
        if let Some(plan) = ktx2_upload_plan(texture, format, bytes) {
            return compressed_plan_readiness(texture, bytes, plan, support);
        }
        return unsupported("ktx2 texture format or level index is not upload-ready");
    }
    if format.starts_with("ktx/") {
        if let Some(plan) = ktx_upload_plan(texture, format, bytes) {
            return compressed_plan_readiness(texture, bytes, plan, support);
        }
        return unsupported("ktx texture format or level payload is not upload-ready");
    }
    unsupported(format!(
        "texture container format {format} is not upload-ready"
    ))
}

fn texture_descriptor_mip_count(texture: &TextureAsset) -> u32 {
    texture.render_image_descriptor().mip_count.max(1)
}

fn texture_descriptor_layer_count(texture: &TextureAsset) -> u32 {
    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension == RenderImageDimension::D3 {
        1
    } else {
        descriptor.array_layer_count.max(1)
    }
}

fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor.max(1)
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
