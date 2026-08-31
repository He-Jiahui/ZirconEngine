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
mod tests {
    use crate::asset::assets::{TextureAsset, TextureAssetDescriptor, TexturePayload};
    use crate::core::framework::render::{
        TextureMetadata, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
    };
    use crate::core::resource::ResourceLocator;

    use super::normalize_texture_normal_map_convention;

    fn rgba8_fixture(
        usage_hint: TextureUsageHint,
        normal_convention: TextureNormalConvention,
    ) -> TextureAsset {
        let mut texture = TextureAsset::new_rgba8(
            ResourceLocator::parse("res://textures/normal-convention.png").unwrap(),
            1,
            1,
            vec![128, 64, 255, 255],
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata = TextureMetadata {
            usage_hint,
            normal_convention,
            mip_policy: TextureMipPolicy::GenerateOffline,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);
        texture
    }

    #[test]
    fn non_normal_payload_returns_without_materializing_a_descriptor() {
        let mut texture = TextureAsset::new_rgba8(
            ResourceLocator::parse("res://textures/albedo.png").unwrap(),
            1,
            1,
            vec![10, 20, 30, 255],
        );
        texture.descriptor = None;

        let normalized = normalize_texture_normal_map_convention(texture.clone()).unwrap();

        assert_eq!(normalized, texture);
        assert!(normalized.descriptor.is_none());
    }

    #[test]
    fn tangent_space_dx_rgba8_flips_green_and_canonicalizes_to_gl() {
        let texture = rgba8_fixture(
            TextureUsageHint::Normal,
            TextureNormalConvention::TangentSpaceDx,
        );

        let normalized = normalize_texture_normal_map_convention(texture).unwrap();

        assert_eq!(normalized.rgba, vec![128, 191, 255, 255]);
        assert_eq!(
            normalized.texture_descriptor().metadata.normal_convention,
            TextureNormalConvention::TangentSpaceGl
        );
    }

    #[test]
    fn tangent_space_gl_rgba8_is_already_canonical() {
        let texture = rgba8_fixture(
            TextureUsageHint::Normal,
            TextureNormalConvention::TangentSpaceGl,
        );

        let normalized = normalize_texture_normal_map_convention(texture).unwrap();

        assert_eq!(normalized.rgba, vec![128, 64, 255, 255]);
        assert_eq!(
            normalized.texture_descriptor().metadata.normal_convention,
            TextureNormalConvention::TangentSpaceGl
        );
    }

    #[test]
    fn compressed_dx_normal_requires_transcode_before_convention_conversion() {
        let uri = ResourceLocator::parse("res://textures/normal.ktx2").unwrap();
        let mut texture =
            TextureAsset::new_container(uri, 4, 4, "ktx2/test".to_string(), vec![0; 16], 1, 1)
                .with_descriptor(TextureAssetDescriptor::container("ktx2/test", 1, 1));
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata.usage_hint = TextureUsageHint::Normal;
        descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceDx;
        texture.descriptor = Some(descriptor);

        let error = normalize_texture_normal_map_convention(texture).unwrap_err();

        assert!(error.to_string().contains("decoded rgba8 payload"));
    }

    #[test]
    fn compressed_gl_normal_is_already_canonical() {
        let uri = ResourceLocator::parse("res://textures/normal.ktx2").unwrap();
        let mut texture =
            TextureAsset::new_container(uri, 4, 4, "ktx2/test".to_string(), vec![7; 16], 1, 1)
                .with_descriptor(TextureAssetDescriptor::container("ktx2/test", 1, 1));
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata.usage_hint = TextureUsageHint::Normal;
        descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceGl;
        texture.descriptor = Some(descriptor);

        let normalized = normalize_texture_normal_map_convention(texture).unwrap();

        assert_eq!(
            normalized.texture_descriptor().metadata.normal_convention,
            TextureNormalConvention::TangentSpaceGl
        );
        match &normalized.payload {
            TexturePayload::Container { bytes, .. } => assert_eq!(bytes.as_slice(), &[7; 16]),
            payload => panic!("expected container payload, got {payload:?}"),
        }
    }
}
