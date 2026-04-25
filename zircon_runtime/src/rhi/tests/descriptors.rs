use crate::rhi::{
    BufferDesc, BufferUsage, PipelineDesc, PipelineKind, SamplerDesc, TextureDesc,
    TextureDimension, TextureFormat, TextureUsage,
};

#[test]
fn resource_descriptors_keep_stable_labels_and_usage() {
    let buffer = BufferDesc::new("frame-uniform", 256, BufferUsage::UNIFORM);
    let texture = TextureDesc::new(
        "scene-color",
        1920,
        1080,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    )
    .with_dimension(TextureDimension::D2);
    let sampler = SamplerDesc::linear("scene-linear");
    let pipeline = PipelineDesc::new("forward-opaque", PipelineKind::Raster);

    assert_eq!(buffer.label.as_deref(), Some("frame-uniform"));
    assert_eq!(buffer.size_bytes, 256);
    assert_eq!(texture.label.as_deref(), Some("scene-color"));
    assert_eq!(texture.width, 1920);
    assert_eq!(texture.height, 1080);
    assert_eq!(texture.dimension, TextureDimension::D2);
    assert!(sampler.linear_filtering);
    assert_eq!(pipeline.kind, PipelineKind::Raster);
}

#[test]
fn rhi_descriptors_do_not_embed_scene_level_semantics() {
    let source = include_str!("../descriptors.rs");
    for forbidden in ["Mesh", "Material", "Light", "Scene"] {
        assert!(
            !source.contains(forbidden),
            "RHI descriptors must not encode upper-layer `{forbidden}` semantics"
        );
    }
}
