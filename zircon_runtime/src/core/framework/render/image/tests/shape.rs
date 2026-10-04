use super::*;
use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageFallbackKind, RenderImageUsage,
    RenderSamplerDescriptor, TextureMetadata,
};

#[test]
fn d2_array_shape_uses_one_validated_layer_count() {
    let mut descriptor = descriptor();
    descriptor.depth_or_array_layers = 4;

    let shape = descriptor.validated_shape().expect("valid d2 array");

    assert_eq!(shape.view_kind, TextureViewKind::D2Array);
    assert_eq!(shape.extent.depth_or_array_layers, 4);
    assert_eq!(shape.array_layer_count(), 4);
    assert_eq!(shape.depth(), 1);
}

#[test]
fn d1_shape_rejects_array_layers() {
    let mut descriptor = descriptor();
    descriptor.dimension = RenderImageDimension::D1;
    descriptor.height = 1;
    descriptor.depth_or_array_layers = 2;

    assert_eq!(
        descriptor.validated_shape(),
        Err(RenderImageShapeError::InvalidD1Extent)
    );
}

#[test]
fn d3_shape_keeps_depth_distinct_from_array_layers() {
    let mut descriptor = descriptor();
    descriptor.dimension = RenderImageDimension::D3;
    descriptor.depth_or_array_layers = 4;

    let shape = descriptor.validated_shape().expect("valid volume");

    assert_eq!(shape.view_kind, TextureViewKind::D3);
    assert_eq!(shape.depth(), 4);
    assert_eq!(shape.array_layer_count(), 1);
}

#[test]
fn cube_array_shape_requires_matching_complete_face_sets() {
    let mut descriptor = descriptor();
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = 12;

    let shape = descriptor.validated_shape().expect("valid cube array");

    assert_eq!(shape.view_kind, TextureViewKind::CubeArray);
    assert_eq!(shape.array_layer_count(), 12);

    descriptor.depth_or_array_layers = 5;
    assert!(matches!(
        descriptor.validated_shape(),
        Err(RenderImageShapeError::CubeLayerCount { layers: 5 })
    ));
}

fn descriptor() -> RenderImageDescriptor {
    RenderImageDescriptor {
        width: 4,
        height: 4,
        depth_or_array_layers: 1,
        dimension: RenderImageDimension::D2,
        format: "rgba8unorm".to_string(),
        color_space: RenderImageColorSpace::Linear,
        metadata: TextureMetadata::default(),
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![RenderImageUsage::Sampled],
        asset_usage: vec![RenderImageAssetUsage::MainWorld],
        mip_count: 1,
        fallback: RenderImageFallbackKind::MissingImage,
    }
}
