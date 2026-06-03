use crate::rhi::{
    BufferDesc, BufferUsage, PipelineDesc, PipelineKind, SamplerDesc, TextureDesc,
    TextureDimension, TextureFormat, TextureResidency, TextureUsage,
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
fn texture_descriptors_cover_hdr_arrays_cubes_mips_and_storage() {
    let array_texture = TextureDesc::new(
        "reflection-probe-array",
        256,
        256,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_DST,
    )
    .with_dimension(TextureDimension::D2Array)
    .with_array_layers(8)
    .with_mip_levels(5);

    assert_eq!(array_texture.dimension, TextureDimension::D2Array);
    assert_eq!(array_texture.depth, 8);
    assert_eq!(array_texture.mip_levels, 5);
    assert!(array_texture.format.is_hdr_color());
    assert!(array_texture.usage.contains(TextureUsage::STORAGE));
    assert_eq!(
        array_texture.checked_storage_size_bytes(),
        Some((256_u64 * 256 + 128 * 128 + 64 * 64 + 32 * 32 + 16 * 16) * 8 * 8)
    );

    let cube_texture = TextureDesc::new(
        "skybox-cubemap",
        512,
        512,
        TextureFormat::Bgra8UnormSrgb,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    )
    .with_dimension(TextureDimension::Cube)
    .with_array_layers(6);

    assert_eq!(cube_texture.dimension, TextureDimension::Cube);
    assert_eq!(cube_texture.depth, 6);
    assert!(!cube_texture.format.is_hdr_color());
    assert_eq!(TextureFormat::Depth24PlusStencil8.bytes_per_pixel(), 4);
}

#[test]
fn texture_descriptors_mark_sparse_reservations_without_losing_virtual_size() {
    let sparse = TextureDesc::new(
        "terrain-virtual-texture",
        16_384,
        16_384,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_DST,
    )
    .with_dimension(TextureDimension::D2Array)
    .with_array_layers(4)
    .with_mip_levels(12)
    .with_sparse_residency();

    assert_eq!(sparse.residency, TextureResidency::SparseReserved);
    assert!(sparse.is_sparse_reserved());
    assert_eq!(
        sparse.checked_storage_size_bytes(),
        Some(
            (16_384_u64 * 16_384
                + 8_192 * 8_192
                + 4_096 * 4_096
                + 2_048 * 2_048
                + 1_024 * 1_024
                + 512 * 512
                + 256 * 256
                + 128 * 128
                + 64 * 64
                + 32 * 32
                + 16 * 16
                + 8 * 8)
                * 4
                * 4
        )
    );
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
