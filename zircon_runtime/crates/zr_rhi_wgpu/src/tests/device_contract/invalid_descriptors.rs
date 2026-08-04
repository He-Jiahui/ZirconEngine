use super::*;

#[test]
fn deterministic_rhi_contract_rejects_invalid_resource_descriptors() {
    let device = DeterministicRhiContractDevice::new_headless();

    assert_eq!(
        device
            .create_buffer(&BufferDesc::new("empty-buffer", 0, BufferUsage::COPY_SRC))
            .unwrap_err(),
        zr_rhi::RhiError::InvalidBufferDescriptor {
            label: Some("empty-buffer".to_string()),
            reason: "size_bytes must be greater than zero".to_string(),
        }
    );
    assert_eq!(
        device
            .create_buffer(&BufferDesc::new("no-buffer-usage", 16, BufferUsage::NONE))
            .unwrap_err(),
        zr_rhi::RhiError::InvalidBufferDescriptor {
            label: Some("no-buffer-usage".to_string()),
            reason: "usage must not be empty".to_string(),
        }
    );

    let zero_extent = TextureDesc::new(
        "zero-extent",
        0,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    );
    assert_eq!(
        device.create_texture(&zero_extent).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("zero-extent".to_string()),
            reason: "width, height, and depth must be greater than zero".to_string(),
        }
    );

    let mut no_mips = TextureDesc::new(
        "no-mips",
        2,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    );
    no_mips.mip_levels = 0;
    assert_eq!(
        device.create_texture(&no_mips).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("no-mips".to_string()),
            reason: "mip_levels must be greater than zero".to_string(),
        }
    );

    let no_usage = TextureDesc::new(
        "no-texture-usage",
        2,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::NONE,
    );
    assert_eq!(
        device.create_texture(&no_usage).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("no-texture-usage".to_string()),
            reason: "usage must not be empty".to_string(),
        }
    );

    let invalid_msaa_mips = TextureDesc::new(
        "invalid-msaa-mips",
        2,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_sample_count(4)
    .with_mip_levels(2);
    assert_eq!(
        device.create_texture(&invalid_msaa_mips).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-msaa-mips".to_string()),
            reason: "multisampled textures cannot declare mip levels".to_string(),
        }
    );

    let invalid_cube_faces = TextureDesc::new(
        "invalid-cube-faces",
        2,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::Cube)
    .with_array_layers(5);
    assert_eq!(
        device.create_texture(&invalid_cube_faces).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-cube-faces".to_string()),
            reason: "cube textures must declare depth as a multiple of six faces".to_string(),
        }
    );

    let invalid_cube_extent = TextureDesc::new(
        "invalid-cube-extent",
        4,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::Cube)
    .with_array_layers(6);
    assert_eq!(
        device.create_texture(&invalid_cube_extent).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-cube-extent".to_string()),
            reason: "cube textures must be square".to_string(),
        }
    );

    let invalid_d1_extent = TextureDesc::new(
        "invalid-d1-extent",
        4,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::D1);
    assert_eq!(
        device.create_texture(&invalid_d1_extent).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-d1-extent".to_string()),
            reason: "1D textures must declare height and depth as 1".to_string(),
        }
    );

    let invalid_d2_depth = TextureDesc::new(
        "invalid-d2-depth",
        4,
        4,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_depth(2);
    assert_eq!(
        device.create_texture(&invalid_d2_depth).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-d2-depth".to_string()),
            reason: "2D textures must declare depth as 1".to_string(),
        }
    );

    let invalid_mip_count = TextureDesc::new(
        "invalid-mip-count",
        4,
        2,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_mip_levels(4);
    assert_eq!(
        device.create_texture(&invalid_mip_count).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-mip-count".to_string()),
            reason: "mip_levels exceeds the texture extent chain".to_string(),
        }
    );

    let invalid_msaa_array = TextureDesc::new(
        "invalid-msaa-array",
        4,
        4,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::D2Array)
    .with_array_layers(2)
    .with_sample_count(4);
    assert_eq!(
        device.create_texture(&invalid_msaa_array).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("invalid-msaa-array".to_string()),
            reason: "multisampling is only valid for 2D textures".to_string(),
        }
    );

    let mut overflowing_storage = TextureDesc::new(
        "overflowing-texture",
        u32::MAX,
        u32::MAX,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::COPY_SRC,
    );
    overflowing_storage.depth = 1;
    assert_eq!(
        device.create_texture(&overflowing_storage).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("overflowing-texture".to_string()),
            reason: "storage size overflows u64".to_string(),
        }
    );

    let invalid_lod_order = SamplerDesc::linear("invalid-lod").with_lod_clamp(4.0, 2.0);
    assert_eq!(
        device.create_sampler(&invalid_lod_order).unwrap_err(),
        zr_rhi::RhiError::InvalidSamplerDescriptor {
            label: Some("invalid-lod".to_string()),
            reason: "lod_min_clamp must be less than or equal to lod_max_clamp".to_string(),
        }
    );

    let invalid_lod_value = SamplerDesc {
        lod_min_clamp: f32::NAN,
        ..SamplerDesc::nearest("invalid-lod-value")
    };
    assert_eq!(
        device.create_sampler(&invalid_lod_value).unwrap_err(),
        zr_rhi::RhiError::InvalidSamplerDescriptor {
            label: Some("invalid-lod-value".to_string()),
            reason: "lod clamps must be finite".to_string(),
        }
    );

    let invalid_anisotropy =
        SamplerDesc::linear_mipmap_linear("invalid-anisotropy").with_anisotropy_clamp(17);
    assert_eq!(
        device.create_sampler(&invalid_anisotropy).unwrap_err(),
        zr_rhi::RhiError::InvalidSamplerDescriptor {
            label: Some("invalid-anisotropy".to_string()),
            reason: "anisotropy_clamp must be in the range 1..=16".to_string(),
        }
    );
}
