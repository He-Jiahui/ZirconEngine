use zircon_runtime::asset::{AssetImportError, TextureAsset, TexturePayload};
use zircon_runtime::core::framework::render::{TextureNormalConvention, TextureUsageHint};

/// Converts decoded normal maps into the engine-wide tangent-space DX convention.
pub(crate) fn normalize_normal_map_convention(
    mut texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    let mut descriptor = texture.texture_descriptor();
    if descriptor.metadata.usage_hint != TextureUsageHint::Normal {
        return Ok(texture);
    }

    match descriptor.metadata.normal_convention {
        TextureNormalConvention::None | TextureNormalConvention::TangentSpaceDx => {
            descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceDx;
        }
        TextureNormalConvention::TangentSpaceGl => {
            if !matches!(&texture.payload, TexturePayload::Rgba8) {
                return Err(AssetImportError::Parse(format!(
                    "normal convention conversion requires a decoded rgba8 payload for {}",
                    texture.uri
                )));
            }
            for texel in texture.rgba.chunks_exact_mut(4) {
                texel[1] = u8::MAX - texel[1];
            }
            descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceDx;
        }
    }

    texture.descriptor = Some(descriptor);
    Ok(texture)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::AssetUri;
    use zircon_runtime::core::framework::render::{TextureMetadata, TextureMipPolicy};

    #[test]
    fn normal_gl_payload_is_converted_to_dx_before_mip_generation() {
        let mut texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/gl-normal.png").expect("valid texture uri"),
            1,
            1,
            vec![128, 64, 255, 255],
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            normal_convention: TextureNormalConvention::TangentSpaceGl,
            mip_policy: TextureMipPolicy::GenerateOffline,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);

        let texture = normalize_normal_map_convention(texture).expect("normal conversion succeeds");

        assert_eq!(texture.rgba, vec![128, 191, 255, 255]);
        assert_eq!(
            texture.texture_descriptor().metadata.normal_convention,
            TextureNormalConvention::TangentSpaceDx
        );
    }
}
