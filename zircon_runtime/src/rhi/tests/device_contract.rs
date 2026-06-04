use crate::rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupHandle,
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindGroupLayoutHandle, BindingResourceType,
    BufferDesc, BufferHandle, BufferUsage, CommandList, CompareFunction, FenceValue, PipelineDesc,
    PipelineHandle, PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle, RenderDevice,
    RenderQueueClass, SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle,
    ShaderStage, TextureDesc, TextureDimension, TextureFormat, TextureHandle, TextureUsage,
};
use crate::rhi_wgpu::WgpuRenderDevice;
use std::path::Path;

#[test]
fn rhi_handles_are_stable_raw_identifiers() {
    assert_eq!(BufferHandle::new(11).raw(), 11);
    assert_eq!(TextureHandle::new(12).raw(), 12);
    assert_eq!(SamplerHandle::new(13).raw(), 13);
    assert_eq!(BindGroupLayoutHandle::new(14).raw(), 14);
    assert_eq!(BindGroupHandle::new(15).raw(), 15);
    assert_eq!(ShaderModuleHandle::new(16).raw(), 16);
    assert_eq!(PipelineLayoutHandle::new(17).raw(), 17);
    assert_eq!(PipelineHandle::new(18).raw(), 18);
}

fn test_bind_group_layout_desc(label: &str) -> BindGroupLayoutDesc {
    BindGroupLayoutDesc::new(
        label,
        vec![BindGroupLayoutEntryDesc::new(
            0,
            BindingResourceType::UniformBuffer,
            vec![
                ShaderStage::Vertex,
                ShaderStage::Fragment,
                ShaderStage::Compute,
            ],
        )],
    )
}

fn create_test_pipeline_layout(device: &WgpuRenderDevice, label: &str) -> PipelineLayoutHandle {
    let bind_group_layout = device
        .create_bind_group_layout(&test_bind_group_layout_desc(&format!("{label}-bind-group")))
        .unwrap();
    device
        .create_pipeline_layout(&PipelineLayoutDesc::new(label, vec![bind_group_layout]))
        .unwrap()
}

#[test]
fn buffer_and_texture_usage_flags_are_composable() {
    let buffer_usage = BufferUsage::UNIFORM | BufferUsage::STORAGE | BufferUsage::COPY_DST;
    assert!(buffer_usage.contains(BufferUsage::UNIFORM));
    assert!(buffer_usage.contains(BufferUsage::STORAGE));
    assert!(buffer_usage.contains(BufferUsage::COPY_DST));
    assert!(!buffer_usage.contains(BufferUsage::INDEX));

    let texture_usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC;
    assert!(texture_usage.contains(TextureUsage::RENDER_ATTACHMENT));
    assert!(texture_usage.contains(TextureUsage::SAMPLED));
    assert!(texture_usage.contains(TextureUsage::COPY_SRC));
    assert!(!texture_usage.contains(TextureUsage::PRESENT));
}

#[test]
fn wgpu_rhi_device_allocates_stable_resource_handles_and_fences() {
    let device = WgpuRenderDevice::new_headless();

    let buffer = device
        .create_buffer(&BufferDesc::new(
            "frame-uniform",
            256,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "scene-color",
            64,
            64,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear("scene-linear"))
        .unwrap();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "fullscreen",
            ShaderStage::Compute,
            "main",
            "@compute @workgroup_size(1) fn main() {}",
        ))
        .unwrap();
    let pipeline_layout = create_test_pipeline_layout(&device, "compute-layout");
    let pipeline = device
        .create_pipeline(
            &PipelineDesc::new("compute", PipelineKind::Compute)
                .with_layout(pipeline_layout)
                .with_compute_shader(shader),
        )
        .unwrap();

    assert_ne!(buffer.raw(), texture.raw());
    assert_ne!(sampler.raw(), shader.raw());
    assert_ne!(pipeline.raw(), 0);

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "copy-upload")
        .unwrap();
    assert_eq!(command_list.queue_class(), RenderQueueClass::Copy);
    assert_eq!(command_list.label(), Some("copy-upload"));
    let compute_command_list = device
        .create_command_list(RenderQueueClass::Compute, "compute-main")
        .unwrap();
    assert_eq!(
        compute_command_list.queue_class(),
        RenderQueueClass::Compute
    );

    let fence = device.submit(command_list).unwrap();
    assert_eq!(fence, FenceValue(1));
    assert!(device.is_fence_complete(fence).unwrap());

    let bytes = device.read_buffer(buffer, 0, 16).unwrap();
    assert_eq!(bytes.len(), 16);

    device.destroy_pipeline(pipeline).unwrap();
    device.destroy_shader_module(shader).unwrap();
    device.destroy_sampler(sampler).unwrap();
    device.destroy_texture(texture).unwrap();
    device.destroy_buffer(buffer).unwrap();
}

#[test]
fn wgpu_rhi_rejects_sparse_reserved_texture_without_backend_support() {
    let device = WgpuRenderDevice::new_headless();
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
            label: Some("virtual-terrain-pages".to_string()),
            reason: "sparse texture residency requires backend sparse texture support".to_string(),
        }
    );
}

#[test]
fn wgpu_rhi_roundtrips_hdr_array_and_cube_texture_descriptors() {
    let device = WgpuRenderDevice::new_headless();
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
fn wgpu_rhi_device_roundtrips_resource_descriptors_by_handle() {
    let device = WgpuRenderDevice::new_headless();
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
        crate::rhi::RhiError::UnknownBuffer(buffer.raw())
    );
}

#[test]
fn wgpu_rhi_roundtrips_shadow_and_trilinear_sampler_descriptors() {
    let device = WgpuRenderDevice::new_headless();
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

#[test]
fn wgpu_rhi_roundtrips_bind_group_layouts_and_bind_groups() {
    let device = WgpuRenderDevice::new_headless();
    let layout_desc = BindGroupLayoutDesc::new(
        "material-layout",
        vec![
            BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::UniformBuffer,
                vec![ShaderStage::Vertex, ShaderStage::Fragment],
            ),
            BindGroupLayoutEntryDesc::new(
                1,
                BindingResourceType::Texture,
                vec![ShaderStage::Fragment],
            ),
            BindGroupLayoutEntryDesc::new(
                2,
                BindingResourceType::Sampler,
                vec![ShaderStage::Fragment],
            ),
        ],
    );
    let uniform = device
        .create_buffer(&BufferDesc::new(
            "material-uniform",
            64,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "albedo",
            4,
            4,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear_mipmap_linear("albedo-sampler"))
        .unwrap();
    let layout = device.create_bind_group_layout(&layout_desc).unwrap();
    let bind_group_desc = BindGroupDesc::new(
        "material-bind-group",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(uniform)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Texture(texture)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );

    let bind_group = device.create_bind_group(&bind_group_desc).unwrap();

    assert_eq!(device.bind_group_layout_desc(layout).unwrap(), layout_desc);
    assert_eq!(device.bind_group_desc(bind_group).unwrap(), bind_group_desc);

    device.destroy_bind_group(bind_group).unwrap();
    assert_eq!(
        device.bind_group_desc(bind_group).unwrap_err(),
        crate::rhi::RhiError::UnknownBindGroup(bind_group.raw())
    );
    device.destroy_bind_group_layout(layout).unwrap();
    assert_eq!(
        device.bind_group_layout_desc(layout).unwrap_err(),
        crate::rhi::RhiError::UnknownBindGroupLayout(layout.raw())
    );
}

#[test]
fn wgpu_rhi_rejects_invalid_bind_group_layout_descriptors() {
    let device = WgpuRenderDevice::new_headless();

    assert_eq!(
        device
            .create_bind_group_layout(&BindGroupLayoutDesc::new("empty-layout", Vec::new()))
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("empty-layout".to_string()),
            reason: "entries must not be empty".to_string(),
        }
    );

    let duplicate_binding = BindGroupLayoutDesc::new(
        "duplicate-binding-layout",
        vec![
            BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::UniformBuffer,
                vec![ShaderStage::Vertex],
            ),
            BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::Sampler,
                vec![ShaderStage::Fragment],
            ),
        ],
    );
    assert_eq!(
        device
            .create_bind_group_layout(&duplicate_binding)
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("duplicate-binding-layout".to_string()),
            reason: "binding 0 is duplicated".to_string(),
        }
    );

    let no_visibility = BindGroupLayoutDesc::new(
        "no-visibility-layout",
        vec![BindGroupLayoutEntryDesc::new(
            2,
            BindingResourceType::Texture,
            Vec::new(),
        )],
    );
    assert_eq!(
        device.create_bind_group_layout(&no_visibility).unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("no-visibility-layout".to_string()),
            reason: "binding 2 has no shader-stage visibility".to_string(),
        }
    );

    let repeated_visibility = BindGroupLayoutDesc::new(
        "repeated-visibility-layout",
        vec![BindGroupLayoutEntryDesc::new(
            3,
            BindingResourceType::StorageBuffer,
            vec![ShaderStage::Compute, ShaderStage::Compute],
        )],
    );
    assert_eq!(
        device
            .create_bind_group_layout(&repeated_visibility)
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("repeated-visibility-layout".to_string()),
            reason: "binding 3 repeats shader-stage visibility".to_string(),
        }
    );
}

#[test]
fn wgpu_rhi_bind_group_validation_checks_layout_resource_types_and_usage() {
    let device = WgpuRenderDevice::new_headless();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "material-layout",
            vec![
                BindGroupLayoutEntryDesc::new(
                    0,
                    BindingResourceType::UniformBuffer,
                    vec![ShaderStage::Vertex, ShaderStage::Fragment],
                ),
                BindGroupLayoutEntryDesc::new(
                    1,
                    BindingResourceType::Texture,
                    vec![ShaderStage::Fragment],
                ),
                BindGroupLayoutEntryDesc::new(
                    2,
                    BindingResourceType::Sampler,
                    vec![ShaderStage::Fragment],
                ),
            ],
        ))
        .unwrap();
    let uniform = device
        .create_buffer(&BufferDesc::new("uniform", 64, BufferUsage::UNIFORM))
        .unwrap();
    let storage_only = device
        .create_buffer(&BufferDesc::new("storage-only", 64, BufferUsage::STORAGE))
        .unwrap();
    let sampled_texture = device
        .create_texture(&TextureDesc::new(
            "sampled",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let storage_texture = device
        .create_texture(&TextureDesc::new(
            "storage",
            2,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        ))
        .unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear("sampled-linear"))
        .unwrap();

    let missing_binding = BindGroupDesc::new(
        "missing-binding",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(uniform)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Texture(sampled_texture)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&missing_binding).unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupDescriptor {
            label: Some("missing-binding".to_string()),
            reason: "entry count 2 does not match layout entry count 3".to_string(),
        }
    );

    let duplicate_binding = BindGroupDesc::new(
        "duplicate-binding",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(uniform)),
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(uniform)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&duplicate_binding).unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupDescriptor {
            label: Some("duplicate-binding".to_string()),
            reason: "binding 0 is duplicated".to_string(),
        }
    );

    let wrong_resource_type = BindGroupDesc::new(
        "wrong-resource-type",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Sampler(sampler)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Texture(sampled_texture)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&wrong_resource_type).unwrap_err(),
        crate::rhi::RhiError::InvalidBindGroupDescriptor {
            label: Some("wrong-resource-type".to_string()),
            reason: format!(
                "binding 0 expects {:?}, got {:?}",
                BindingResourceType::UniformBuffer,
                BindGroupEntryResource::Sampler(sampler)
            ),
        }
    );

    let invalid_buffer_usage = BindGroupDesc::new(
        "invalid-buffer-usage",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(storage_only)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Texture(sampled_texture)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&invalid_buffer_usage).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: storage_only.raw(),
            required: BufferUsage::UNIFORM,
            actual: BufferUsage::STORAGE,
        }
    );

    let invalid_texture_usage = BindGroupDesc::new(
        "invalid-texture-usage",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(uniform)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Texture(storage_texture)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device
            .create_bind_group(&invalid_texture_usage)
            .unwrap_err(),
        crate::rhi::RhiError::InvalidTextureUsage {
            texture: storage_texture.raw(),
            required: TextureUsage::SAMPLED,
            actual: TextureUsage::STORAGE,
        }
    );

    let unknown_sampler = BindGroupDesc::new(
        "unknown-sampler",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Buffer(uniform)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Texture(sampled_texture)),
            BindGroupEntryDesc::new(
                2,
                BindGroupEntryResource::Sampler(SamplerHandle::new(9_999)),
            ),
        ],
    );
    assert_eq!(
        device.create_bind_group(&unknown_sampler).unwrap_err(),
        crate::rhi::RhiError::UnknownSampler(9_999)
    );
}

#[test]
fn wgpu_rhi_rejects_invalid_resource_descriptors() {
    let device = WgpuRenderDevice::new_headless();

    assert_eq!(
        device
            .create_buffer(&BufferDesc::new("empty-buffer", 0, BufferUsage::COPY_SRC))
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBufferDescriptor {
            label: Some("empty-buffer".to_string()),
            reason: "size_bytes must be greater than zero".to_string(),
        }
    );
    assert_eq!(
        device
            .create_buffer(&BufferDesc::new("no-buffer-usage", 16, BufferUsage::NONE))
            .unwrap_err(),
        crate::rhi::RhiError::InvalidBufferDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
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
        crate::rhi::RhiError::InvalidTextureDescriptor {
            label: Some("overflowing-texture".to_string()),
            reason: "storage size overflows u64".to_string(),
        }
    );

    let invalid_lod_order = SamplerDesc::linear("invalid-lod").with_lod_clamp(4.0, 2.0);
    assert_eq!(
        device.create_sampler(&invalid_lod_order).unwrap_err(),
        crate::rhi::RhiError::InvalidSamplerDescriptor {
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
        crate::rhi::RhiError::InvalidSamplerDescriptor {
            label: Some("invalid-lod-value".to_string()),
            reason: "lod clamps must be finite".to_string(),
        }
    );

    let invalid_anisotropy =
        SamplerDesc::linear_mipmap_linear("invalid-anisotropy").with_anisotropy_clamp(17);
    assert_eq!(
        device.create_sampler(&invalid_anisotropy).unwrap_err(),
        crate::rhi::RhiError::InvalidSamplerDescriptor {
            label: Some("invalid-anisotropy".to_string()),
            reason: "anisotropy_clamp must be in the range 1..=16".to_string(),
        }
    );
}

#[test]
fn wgpu_rhi_fence_queries_reject_unissued_fence_values() {
    let device = WgpuRenderDevice::new_headless();

    assert_eq!(
        device.is_fence_complete(FenceValue(0)).unwrap_err(),
        crate::rhi::RhiError::UnknownFence(0)
    );
    assert_eq!(
        device.is_fence_complete(FenceValue(7)).unwrap_err(),
        crate::rhi::RhiError::UnknownFence(7)
    );

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "empty-copy")
        .unwrap();
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());
    assert_eq!(
        device
            .is_fence_complete(FenceValue(fence.0 + 1))
            .unwrap_err(),
        crate::rhi::RhiError::UnknownFence(fence.0 + 1)
    );
}

#[test]
fn wgpu_rhi_write_copy_and_read_buffer_preserves_bytes() {
    let device = WgpuRenderDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let gpu_buffer = device
        .create_buffer(&BufferDesc::new(
            "gpu-buffer",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    device
        .write_buffer(upload, 4, &[10, 20, 30, 40, 50, 60])
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "upload-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(upload, gpu_buffer, 4, 2, 6);
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    assert_eq!(
        device.read_buffer(gpu_buffer, 0, 10).unwrap(),
        vec![0, 0, 10, 20, 30, 40, 50, 60, 0, 0]
    );
}

#[test]
fn wgpu_rhi_write_buffer_validates_usage_and_range() {
    let device = WgpuRenderDevice::new_headless();
    let read_only = device
        .create_buffer(&BufferDesc::new("read-only", 8, BufferUsage::STAGING_READ))
        .unwrap();

    assert_eq!(
        device.write_buffer(read_only, 0, &[1, 2, 3]).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: read_only.raw(),
            required: BufferUsage::STAGING_WRITE,
            actual: BufferUsage::STAGING_READ,
        }
    );

    let upload = device
        .create_buffer(&BufferDesc::new("upload", 8, BufferUsage::STAGING_WRITE))
        .unwrap();
    assert_eq!(
        device.write_buffer(upload, 6, &[1, 2, 3]).unwrap_err(),
        crate::rhi::RhiError::WriteOutOfRange {
            buffer: upload.raw(),
            offset: 6,
            size: 3,
        }
    );
}

#[test]
fn wgpu_rhi_read_texture_validates_usage() {
    let device = WgpuRenderDevice::new_headless();
    let write_only = device
        .create_texture(&TextureDesc::new(
            "write-only-texture",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    assert_eq!(
        device.read_texture(write_only).unwrap_err(),
        crate::rhi::RhiError::InvalidTextureUsage {
            texture: write_only.raw(),
            required: TextureUsage::COPY_SRC,
            actual: TextureUsage::COPY_DST,
        }
    );
}

#[test]
fn wgpu_rhi_read_buffer_validates_usage_and_range() {
    let device = WgpuRenderDevice::new_headless();
    let non_readback = device
        .create_buffer(&BufferDesc::new("non-readback", 8, BufferUsage::COPY_DST))
        .unwrap();

    assert_eq!(
        device.read_buffer(non_readback, 0, 4).unwrap_err(),
        crate::rhi::RhiError::InvalidBufferUsage {
            buffer: non_readback.raw(),
            required: BufferUsage::STAGING_READ,
            actual: BufferUsage::COPY_DST,
        }
    );

    let readback = device
        .create_buffer(&BufferDesc::new("readback", 8, BufferUsage::STAGING_READ))
        .unwrap();
    assert_eq!(
        device.read_buffer(readback, 6, 3).unwrap_err(),
        crate::rhi::RhiError::ReadbackOutOfRange {
            buffer: readback.raw(),
            offset: 6,
            size: 3,
        }
    );
}

#[test]
fn app_editor_and_core_framework_sources_do_not_import_wgpu() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime should live under the workspace root");
    let boundary_roots = [
        runtime_root.join("src").join("core").join("framework"),
        workspace_root.join("zircon_app").join("src"),
        workspace_root.join("zircon_editor").join("src"),
    ];
    let mut offenders = Vec::new();
    for root in boundary_roots {
        collect_wgpu_imports(&root, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "app/editor/framework sources must stay behind RenderFramework/RHI boundaries: {offenders:?}"
    );
}

fn collect_wgpu_imports(path: &Path, offenders: &mut Vec<String>) {
    let entries = std::fs::read_dir(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_wgpu_imports(&path, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (line_index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            let imports_wgpu = trimmed.starts_with("use wgpu")
                || trimmed.starts_with("use ::wgpu")
                || (trimmed.contains("wgpu::") && !trimmed.contains('"'));
            if imports_wgpu {
                offenders.push(format!("{}:{}", path.display(), line_index + 1));
            }
        }
    }
}
