use crate::core::framework::render::RenderMaterialTextureTransform;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 纹理槽的 UV 选择与可选变换必须一起传给几何及材质投影。
/// `transform = None` 仅表示恒等变换，不能丢弃独立选择的 UV 频道。
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
#[path = "tests/gltf_texture_transform.rs"]
mod tests;
