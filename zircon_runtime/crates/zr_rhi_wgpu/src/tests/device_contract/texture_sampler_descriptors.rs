use super::*;

#[test]
fn deterministic_rhi_contract_rejects_sparse_reserved_texture_without_backend_support() {
    let device = DeterministicRhiContractDevice::new_headless();
    let sparse = TextureDesc::new(
        "virtual-terrain-pages",
        4096,
        4096,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_DST,
    )
    .with_mip_levels(8)
    .with_sparse_residency();

    assert_eq!(
        device.create_texture(&sparse).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureDescriptor {
            label: Some("virtual-terrain-pages".to_string()),
            reason: "sparse texture residency requires backend sparse texture support".to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_roundtrips_hdr_array_and_cube_texture_descriptors() {
    let device = DeterministicRhiContractDevice::new_headless();
    let hdr_array = TextureDesc::new(
        "hdr-array",
        16,
        16,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::D2Array)
    .with_array_layers(4)
    .with_mip_levels(3);
    let cube = TextureDesc::new(
        "skybox-cube",
        8,
        8,
        TextureFormat::Bgra8UnormSrgb,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    )
    .with_dimension(TextureDimension::Cube)
    .with_array_layers(6);

    let hdr_array_handle = device.create_texture(&hdr_array).unwrap();
    let cube_handle = device.create_texture(&cube).unwrap();

    assert_eq!(device.texture_desc(hdr_array_handle).unwrap(), hdr_array);
    assert_eq!(
        device.read_texture(hdr_array_handle).unwrap().len() as u64,
        hdr_array.checked_storage_size_bytes().unwrap()
    );
    assert_eq!(device.texture_desc(cube_handle).unwrap(), cube);
}

#[test]
fn deterministic_rhi_contract_device_roundtrips_resource_descriptors_by_handle() {
    let device = DeterministicRhiContractDevice::new_headless();
    let buffer_desc = BufferDesc::new(
        "frame-uniform",
        256,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    );
    let texture_desc = TextureDesc::new(
        "scene-color",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );
    let sampler_desc = SamplerDesc::linear("scene-linear");
    let shader_desc = ShaderModuleDesc::new(
        "fullscreen",
        ShaderStage::Compute,
        "main",
        "@compute @workgroup_size(1) fn main() {}",
    );

    let buffer = device.create_buffer(&buffer_desc).unwrap();
    let texture = device.create_texture(&texture_desc).unwrap();
    let sampler = device.create_sampler(&sampler_desc).unwrap();
    let shader = device.create_shader_module(&shader_desc).unwrap();
    let pipeline_layout = create_test_pipeline_layout(&device, "compute-layout");
    let pipeline_desc = PipelineDesc::new("compute", PipelineKind::Compute)
        .with_layout(pipeline_layout)
        .with_compute_shader(shader);
    let pipeline = device.create_pipeline(&pipeline_desc).unwrap();

    assert_eq!(device.buffer_desc(buffer).unwrap(), buffer_desc);
    assert_eq!(device.texture_desc(texture).unwrap(), texture_desc);
    assert_eq!(device.sampler_desc(sampler).unwrap(), sampler_desc);
    assert_eq!(device.shader_module_desc(shader).unwrap(), shader_desc);
    assert_eq!(device.pipeline_desc(pipeline).unwrap(), pipeline_desc);

    device.destroy_buffer(buffer).unwrap();
    assert_eq!(
        device.buffer_desc(buffer).unwrap_err(),
        zr_rhi::RhiError::UnknownBuffer(buffer.raw())
    );
}

#[test]
fn deterministic_rhi_contract_roundtrips_shadow_and_trilinear_sampler_descriptors() {
    let device = DeterministicRhiContractDevice::new_headless();
    let trilinear = SamplerDesc::linear_mipmap_linear("material-trilinear")
        .with_lod_clamp(0.0, 12.0)
        .with_anisotropy_clamp(16);
    let shadow = SamplerDesc::nearest("shadow-map")
        .with_compare(CompareFunction::LessEqual)
        .with_lod_clamp(0.0, 0.0);

    let trilinear_handle = device.create_sampler(&trilinear).unwrap();
    let shadow_handle = device.create_sampler(&shadow).unwrap();

    assert_eq!(device.sampler_desc(trilinear_handle).unwrap(), trilinear);
    assert_eq!(device.sampler_desc(shadow_handle).unwrap(), shadow);
}
