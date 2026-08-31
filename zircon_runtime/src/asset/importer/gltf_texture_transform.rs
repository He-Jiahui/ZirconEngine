use crate::core::framework::render::RenderMaterialTextureTransform;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GltfTextureTransformProjection {
    pub transform: Option<RenderMaterialTextureTransform>,
    pub uv_channel: u32,
}

/// Projects raw `KHR_texture_transform` JSON shared by glTF material importers.
pub fn project_gltf_texture_transform(
    fallback_uv_channel: u32,
    extension: Option<&serde_json::Value>,
) -> GltfTextureTransformProjection {
    let Some(extension) = extension else {
        return GltfTextureTransformProjection {
            transform: None,
            uv_channel: fallback_uv_channel,
        };
    };
    let uv_channel = extension
        .get("texCoord")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(fallback_uv_channel);
    let transform = RenderMaterialTextureTransform {
        scale: extension
            .get("scale")
            .and_then(json_vec2)
            .unwrap_or(RenderMaterialTextureTransform::IDENTITY.scale),
        offset: extension
            .get("offset")
            .and_then(json_vec2)
            .unwrap_or(RenderMaterialTextureTransform::IDENTITY.offset),
        rotation: extension
            .get("rotation")
            .and_then(json_f32)
            .unwrap_or(RenderMaterialTextureTransform::IDENTITY.rotation),
    };
    GltfTextureTransformProjection {
        transform: (!transform.is_identity()).then_some(transform),
        uv_channel,
    }
}

fn json_vec2(value: &serde_json::Value) -> Option<[f32; 2]> {
    let [x, y] = value.as_array()?.as_slice() else {
        return None;
    };
    Some([json_f32(x)?, json_f32(y)?])
}

fn json_f32(value: &serde_json::Value) -> Option<f32> {
    let value = value.as_f64()? as f32;
    value.is_finite().then_some(value)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::project_gltf_texture_transform;
    use crate::core::framework::render::RenderMaterialTextureTransform;

    #[test]
    fn gltf_texture_transform_projection_preserves_scale_offset_rotation_and_texcoord() {
        let extension = json!({
            "texCoord": 1,
            "scale": [2.0, 0.5],
            "offset": [0.25, -0.75],
            "rotation": 1.5707964,
        });

        let projection = project_gltf_texture_transform(0, Some(&extension));

        assert_eq!(projection.uv_channel, 1);
        assert_eq!(
            projection.transform,
            Some(RenderMaterialTextureTransform {
                scale: [2.0, 0.5],
                offset: [0.25, -0.75],
                rotation: 1.5707964,
            })
        );
    }

    #[test]
    fn gltf_texture_transform_projection_keeps_fallback_texcoord_and_field_defaults() {
        let extension = json!({
            "texCoord": -1,
            "scale": [2.0],
            "offset": ["invalid", 0.25],
            "rotation": 0.25,
        });

        let projection = project_gltf_texture_transform(1, Some(&extension));

        assert_eq!(projection.uv_channel, 1);
        assert_eq!(
            projection.transform,
            Some(RenderMaterialTextureTransform {
                scale: RenderMaterialTextureTransform::IDENTITY.scale,
                offset: RenderMaterialTextureTransform::IDENTITY.offset,
                rotation: 0.25,
            }),
            "malformed fields must independently preserve their identity defaults"
        );
    }

    #[test]
    fn gltf_texture_transform_projection_rejects_surplus_vec2_components() {
        let extension = json!({
            "scale": [2.0, 0.5, 0.25],
            "rotation": 0.25,
        });

        let projection = project_gltf_texture_transform(0, Some(&extension));

        assert_eq!(
            projection.transform,
            Some(RenderMaterialTextureTransform {
                scale: RenderMaterialTextureTransform::IDENTITY.scale,
                offset: RenderMaterialTextureTransform::IDENTITY.offset,
                rotation: 0.25,
            })
        );
    }

    #[test]
    fn gltf_texture_transform_projection_preserves_fallback_without_extension() {
        let projection = project_gltf_texture_transform(1, None);

        assert_eq!(projection.uv_channel, 1);
        assert_eq!(projection.transform, None);
    }

    #[test]
    fn gltf_texture_transform_projection_rejects_values_outside_the_f32_domain() {
        let extension = json!({
            "scale": [1e100, 1.0],
        });

        let projection = project_gltf_texture_transform(0, Some(&extension));

        assert_eq!(projection.uv_channel, 0);
        assert_eq!(projection.transform, None);
    }
}
