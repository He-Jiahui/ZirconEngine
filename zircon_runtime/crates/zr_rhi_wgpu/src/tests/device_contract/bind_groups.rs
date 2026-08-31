use super::*;

fn filterable_d2_texture_binding() -> BindingResourceType {
    BindingResourceType::SampledTexture {
        sample_type: TextureSampleType::Float { filterable: true },
        view_dimension: TextureViewDimension::D2,
        multisampled: false,
    }
}

#[test]
fn deterministic_rhi_contract_roundtrips_bind_group_layouts_and_bind_groups() {
    let device = DeterministicRhiContractDevice::new_headless();
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
                filterable_d2_texture_binding(),
                vec![ShaderStage::Fragment],
            ),
            BindGroupLayoutEntryDesc::new(
                2,
                BindingResourceType::Sampler(SamplerBindingType::Filtering),
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
    let texture_view = device
        .create_texture_view(&TextureViewDesc::new(
            "albedo-view",
            texture,
            TextureViewDimension::D2,
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
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            ),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::TextureView(texture_view)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );

    let bind_group = device.create_bind_group(&bind_group_desc).unwrap();

    assert_eq!(device.bind_group_layout_desc(layout).unwrap(), layout_desc);
    assert_eq!(device.bind_group_desc(bind_group).unwrap(), bind_group_desc);

    device.destroy_bind_group(bind_group).unwrap();
    assert_eq!(
        device.bind_group_desc(bind_group).unwrap_err(),
        zr_rhi::RhiError::UnknownBindGroup(bind_group.diagnostic_id())
    );
    device.destroy_bind_group_layout(layout).unwrap();
    assert_eq!(
        device.bind_group_layout_desc(layout).unwrap_err(),
        zr_rhi::RhiError::UnknownBindGroupLayout(layout.diagnostic_id())
    );
}

#[test]
fn deterministic_rhi_contract_preserves_explicit_dynamic_buffer_binding_ranges() {
    let device = DeterministicRhiContractDevice::new_headless();
    let layout_desc = BindGroupLayoutDesc::new(
        "dynamic-uniform-layout",
        vec![BindGroupLayoutEntryDesc::new(
            0,
            BindingResourceType::UniformBuffer,
            vec![ShaderStage::Vertex],
        )
        .with_dynamic_offset()
        .with_min_binding_size(64)],
    );
    let uniform = device
        .create_buffer(&BufferDesc::new(
            "dynamic-uniform",
            512,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let layout = device.create_bind_group_layout(&layout_desc).unwrap();
    let binding = zr_rhi::BindGroupBufferBinding::new(uniform, 64, Some(128));
    let bind_group_desc = BindGroupDesc::new(
        "dynamic-uniform-bind-group",
        layout,
        vec![BindGroupEntryDesc::new(
            0,
            BindGroupEntryResource::Buffer(binding),
        )],
    );

    let bind_group = device.create_bind_group(&bind_group_desc).unwrap();

    assert_eq!(device.bind_group_layout_desc(layout).unwrap(), layout_desc);
    assert_eq!(device.bind_group_desc(bind_group).unwrap(), bind_group_desc);
}

#[test]
fn deterministic_rhi_contract_rejects_buffer_ranges_below_layout_minimum() {
    let device = DeterministicRhiContractDevice::new_headless();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "minimum-buffer-range-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::UniformBuffer,
                vec![ShaderStage::Compute],
            )
            .with_min_binding_size(65)],
        ))
        .unwrap();
    let uniform = device
        .create_buffer(&BufferDesc::new(
            "minimum-buffer-range-uniform",
            64,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let desc = BindGroupDesc::new(
        "undersized-buffer-range",
        layout,
        vec![BindGroupEntryDesc::new(
            0,
            BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::new(
                uniform,
                0,
                Some(64),
            )),
        )],
    );

    assert_eq!(
        device.create_bind_group(&desc).unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupDescriptor {
            label: Some("undersized-buffer-range".to_string()),
            reason: "binding 0 binds 64 bytes, below layout minimum 65".to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_rejects_invalid_bind_group_layout_descriptors() {
    let device = DeterministicRhiContractDevice::new_headless();

    assert_eq!(
        device
            .create_bind_group_layout(&BindGroupLayoutDesc::new("empty-layout", Vec::new()))
            .unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupLayoutDescriptor {
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
                BindingResourceType::Sampler(SamplerBindingType::Filtering),
                vec![ShaderStage::Fragment],
            ),
        ],
    );
    assert_eq!(
        device
            .create_bind_group_layout(&duplicate_binding)
            .unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("duplicate-binding-layout".to_string()),
            reason: "binding 0 is duplicated".to_string(),
        }
    );

    let no_visibility = BindGroupLayoutDesc::new(
        "no-visibility-layout",
        vec![BindGroupLayoutEntryDesc::new(
            2,
            filterable_d2_texture_binding(),
            Vec::new(),
        )],
    );
    assert_eq!(
        device.create_bind_group_layout(&no_visibility).unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupLayoutDescriptor {
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
        zr_rhi::RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("repeated-visibility-layout".to_string()),
            reason: "binding 3 repeats shader-stage visibility".to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_bind_group_validation_checks_layout_resource_types_and_usage() {
    let device = DeterministicRhiContractDevice::new_headless();
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
                    filterable_d2_texture_binding(),
                    vec![ShaderStage::Fragment],
                ),
                BindGroupLayoutEntryDesc::new(
                    2,
                    BindingResourceType::Sampler(SamplerBindingType::Filtering),
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
    let sampled_texture_view = device
        .create_texture_view(&TextureViewDesc::new(
            "sampled-view",
            sampled_texture,
            TextureViewDimension::D2,
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
    let storage_texture_view = device
        .create_texture_view(&TextureViewDesc::new(
            "storage-view",
            storage_texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear("sampled-linear"))
        .unwrap();

    let missing_binding = BindGroupDesc::new(
        "missing-binding",
        layout,
        vec![
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            ),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::TextureView(sampled_texture_view)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&missing_binding).unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupDescriptor {
            label: Some("missing-binding".to_string()),
            reason: "entry count 2 does not match layout entry count 3".to_string(),
        }
    );

    let duplicate_binding = BindGroupDesc::new(
        "duplicate-binding",
        layout,
        vec![
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            ),
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            ),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&duplicate_binding).unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupDescriptor {
            label: Some("duplicate-binding".to_string()),
            reason: "binding 0 is duplicated".to_string(),
        }
    );

    let wrong_resource_type = BindGroupDesc::new(
        "wrong-resource-type",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::Sampler(sampler)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::TextureView(sampled_texture_view)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&wrong_resource_type).unwrap_err(),
        zr_rhi::RhiError::InvalidBindGroupDescriptor {
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
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(storage_only)),
            ),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::TextureView(sampled_texture_view)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&invalid_buffer_usage).unwrap_err(),
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: storage_only.diagnostic_id(),
            required: BufferUsage::UNIFORM,
            actual: BufferUsage::STORAGE,
        }
    );

    let invalid_texture_usage = BindGroupDesc::new(
        "invalid-texture-usage",
        layout,
        vec![
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            ),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::TextureView(storage_texture_view)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(sampler)),
        ],
    );
    assert_eq!(
        device
            .create_bind_group(&invalid_texture_usage)
            .unwrap_err(),
        zr_rhi::RhiError::InvalidTextureUsage {
            texture: storage_texture.diagnostic_id(),
            required: TextureUsage::SAMPLED,
            actual: TextureUsage::STORAGE,
        }
    );

    let stale_sampler = device
        .create_sampler(&SamplerDesc::linear("stale-sampler"))
        .unwrap();
    device.destroy_sampler(stale_sampler).unwrap();
    let unknown_sampler = BindGroupDesc::new(
        "unknown-sampler",
        layout,
        vec![
            BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(uniform)),
            ),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::TextureView(sampled_texture_view)),
            BindGroupEntryDesc::new(2, BindGroupEntryResource::Sampler(stale_sampler)),
        ],
    );
    assert_eq!(
        device.create_bind_group(&unknown_sampler).unwrap_err(),
        zr_rhi::RhiError::UnknownSampler(stale_sampler.diagnostic_id())
    );
}
