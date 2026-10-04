//! 法线图方向在导入后统一为图形侧的约定；压缩数据不能原地翻转通道，调用方需先转码或保留明确的失败。

use std::fmt;

use crate::core::framework::render::{TextureNormalConvention, TextureUsageHint};

use super::{TextureAsset, TexturePayload};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureNormalConventionError {
    message: String,
}

impl fmt::Display for TextureNormalConventionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TextureNormalConventionError {}

/// Converts decoded normal maps into the engine-wide right-handed tangent-space GL convention.
pub fn normalize_texture_normal_map_convention(
    mut texture: TextureAsset,
) -> Result<TextureAsset, TextureNormalConventionError> {
    let usage_hint = texture
        .descriptor
        .as_ref()
        .map(|descriptor| descriptor.metadata.usage_hint)
        .unwrap_or_else(|| texture.texture_descriptor().metadata.usage_hint);
    if usage_hint != TextureUsageHint::Normal {
        return Ok(texture);
    }

    let mut descriptor = texture
        .descriptor
        .take()
        .unwrap_or_else(|| texture.texture_descriptor())
        .normalized();
    match descriptor.metadata.normal_convention {
        TextureNormalConvention::None | TextureNormalConvention::TangentSpaceGl => {
            descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceGl;
        }
        TextureNormalConvention::TangentSpaceDx => {
            if !matches!(&texture.payload, TexturePayload::Rgba8) {
                return Err(TextureNormalConventionError {
                    message: format!(
                        "normal convention conversion requires a decoded rgba8 payload for {}",
                        texture.uri
                    ),
                });
            }
            for texel in texture.rgba.chunks_exact_mut(4) {
                texel[1] = u8::MAX - texel[1];
            }
            descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceGl;
        }
    }
    texture.descriptor = Some(descriptor);
    Ok(texture)
}

#[cfg(test)]
#[path = "tests/normal_convention.rs"]
mod tests;
