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
