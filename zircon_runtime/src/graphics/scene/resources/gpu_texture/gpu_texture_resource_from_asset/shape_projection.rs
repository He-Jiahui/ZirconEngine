use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageShape, TextureMipPolicy,
    TextureViewKind,
};
use crate::graphics::types::GraphicsError;

/// 在创建纹理或视图前校验存储形状，并把失败连同资产 URI 返回；各上传路径据此选择维度、层数及视图范围。
pub(super) fn validated_texture_shape(
    descriptor: &RenderImageDescriptor,
    texture_uri: &(impl std::fmt::Display + ?Sized),
) -> Result<RenderImageShape, GraphicsError> {
    descriptor.validated_shape().map_err(|error| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} has invalid storage/view shape metadata: {error}"
        ))
    })
}

pub(super) fn wgpu_dimension(view_kind: TextureViewKind) -> wgpu::TextureDimension {
    match view_kind {
        TextureViewKind::D1 => wgpu::TextureDimension::D1,
        TextureViewKind::D3 => wgpu::TextureDimension::D3,
        TextureViewKind::D2
        | TextureViewKind::D2Array
        | TextureViewKind::Cube
        | TextureViewKind::CubeArray => wgpu::TextureDimension::D2,
    }
}

pub(super) fn texture_view_descriptor(
    descriptor: &RenderImageDescriptor,
    shape: RenderImageShape,
) -> wgpu::TextureViewDescriptor<'static> {
    let format = (descriptor.metadata.mip_policy == TextureMipPolicy::GenerateRuntime
        && descriptor.metadata.color_space == RenderImageColorSpace::Srgb)
        .then_some(wgpu::TextureFormat::Rgba8UnormSrgb);
    match shape.view_kind {
        TextureViewKind::D1 => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        },
        TextureViewKind::D2 => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: 0,
            array_layer_count: Some(1),
            ..Default::default()
        },
        TextureViewKind::D2Array => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_array_layer: 0,
            array_layer_count: Some(shape.array_layer_count()),
            ..Default::default()
        },
        TextureViewKind::D3 => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        },
        TextureViewKind::Cube => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            base_array_layer: 0,
            array_layer_count: Some(shape.array_layer_count()),
            ..Default::default()
        },
        TextureViewKind::CubeArray => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::CubeArray),
            base_array_layer: 0,
            array_layer_count: Some(shape.array_layer_count()),
            ..Default::default()
        },
    }
}

pub(super) fn lightmap_texture_view_descriptor(
    layer_count: u32,
) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        base_array_layer: 0,
        array_layer_count: Some(layer_count.max(1)),
        ..Default::default()
    }
}

pub(super) fn lightmap_page_zero_bind_group_view_descriptor() -> wgpu::TextureViewDescriptor<'static>
{
    wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        base_array_layer: 0,
        array_layer_count: Some(1),
        ..Default::default()
    }
}
