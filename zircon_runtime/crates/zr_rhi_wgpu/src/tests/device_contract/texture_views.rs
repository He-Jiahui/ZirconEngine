use super::*;
use zr_rhi::StorageTextureBindingDesc;

#[test]
fn deterministic_rhi_contract_binds_typed_sampled_texture_views() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "material-array",
                8,
                8,
                TextureFormat::Rgba8UnormSrgb,
                TextureUsage::SAMPLED,
            )
            .with_dimension(TextureDimension::D2Array)
            .with_array_layers(2)
            .with_mip_levels(2),
        )
        .unwrap();
    let view_desc =
        TextureViewDesc::new("material-array-srv", texture, TextureViewDimension::D2Array)
            .with_mip_range(0, 2)
            .with_array_layer_range(0, 2);
    let view = device.create_texture_view(&view_desc).unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear_mipmap_linear("material-sampler"))
        .unwrap();
    let layout_desc = BindGroupLayoutDesc::new(
        "material-layout",
        vec![
            BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::SampledTexture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2Array,
                    multisampled: false,
                },
                vec![ShaderStage::Fragment],
            ),
            BindGroupLayoutEntryDesc::new(
                1,
                BindingResourceType::Sampler(SamplerBindingType::Filtering),
                vec![ShaderStage::Fragment],
            ),
        ],
    );
    let layout = device.create_bind_group_layout(&layout_desc).unwrap();
    let bind_group_desc = BindGroupDesc::new(
        "material-bind-group",
        layout,
        vec![
            BindGroupEntryDesc::new(0, BindGroupEntryResource::TextureView(view)),
            BindGroupEntryDesc::new(1, BindGroupEntryResource::Sampler(sampler)),
        ],
    );

    let bind_group = device.create_bind_group(&bind_group_desc).unwrap();

    assert_eq!(device.texture_view_desc(view).unwrap(), view_desc);
    assert_eq!(device.bind_group_desc(bind_group).unwrap(), bind_group_desc);
}

#[test]
fn deterministic_rhi_contract_rejects_invalid_texture_view_ranges_and_parent_destruction() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "cube-array",
                8,
                8,
                TextureFormat::Rgba8Unorm,
                TextureUsage::SAMPLED,
            )
            .with_dimension(TextureDimension::Cube)
            .with_array_layers(12)
            .with_mip_levels(2),
        )
        .unwrap();
    let invalid_view = TextureViewDesc::new("misaligned-cube", texture, TextureViewDimension::Cube)
        .with_array_layer_range(1, 6);
    assert_eq!(
        device.create_texture_view(&invalid_view).unwrap_err(),
        RhiError::InvalidTextureViewDescriptor {
            label: Some("misaligned-cube".to_string()),
            reason: "cube views must start on a six-face boundary".to_string(),
        }
    );

    let view = device
        .create_texture_view(
            &TextureViewDesc::new("cube-srv", texture, TextureViewDimension::Cube)
                .with_array_layer_range(0, 6),
        )
        .unwrap();
    let second_view = device
        .create_texture_view(
            &TextureViewDesc::new("cube-srv-second", texture, TextureViewDimension::Cube)
                .with_array_layer_range(6, 6),
        )
        .unwrap();
    assert_eq!(
        device.destroy_texture(texture).unwrap_err(),
        RhiError::TextureHasLiveViews {
            texture: texture.diagnostic_id(),
            live_views: 2,
        }
    );

    device.destroy_texture_view(view).unwrap();
    assert_eq!(
        device.destroy_texture(texture).unwrap_err(),
        RhiError::TextureHasLiveViews {
            texture: texture.diagnostic_id(),
            live_views: 1,
        }
    );
    device.destroy_texture_view(second_view).unwrap();
    device.destroy_texture(texture).unwrap();
    assert_eq!(
        device.texture_view_desc(view).unwrap_err(),
        RhiError::UnknownTextureView(view.diagnostic_id())
    );
}

#[test]
fn deterministic_rhi_contract_rejects_filterable_layout_for_unfilterable_float_texture() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(&TextureDesc::new(
            "depth-pyramid",
            4,
            4,
            TextureFormat::R32Float,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let view = device
        .create_texture_view(&TextureViewDesc::new(
            "depth-pyramid-srv",
            texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "filterable-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::SampledTexture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();

    let error = device
        .create_bind_group(&BindGroupDesc::new(
            "filterable-bind-group",
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(view),
            )],
        ))
        .unwrap_err();
    assert_eq!(
        error,
        RhiError::InvalidBindGroupDescriptor {
            label: Some("filterable-bind-group".to_string()),
            reason: "binding 0 requires a filterable float texture view".to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_binds_typed_write_only_storage_texture_views() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(&TextureDesc::new(
            "mip-generation-target",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        ))
        .unwrap();
    let view = device
        .create_texture_view(&TextureViewDesc::new(
            "mip-generation-target-uav",
            texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let storage =
        StorageTextureBindingDesc::write_only(TextureFormat::Rgba8Unorm, TextureViewDimension::D2);
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "mip-generation-storage-layout",
            vec![BindGroupLayoutEntryDesc::new(
                3,
                BindingResourceType::StorageTexture(storage),
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();
    let bind_group = BindGroupDesc::new(
        "mip-generation-storage-bind-group",
        layout,
        vec![BindGroupEntryDesc::new(
            3,
            BindGroupEntryResource::TextureView(view),
        )],
    );
    device.create_bind_group(&bind_group).unwrap();

    let incompatible_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "incompatible-storage-layout",
            vec![BindGroupLayoutEntryDesc::new(
                3,
                BindingResourceType::StorageTexture(StorageTextureBindingDesc::write_only(
                    TextureFormat::Rgba16Float,
                    TextureViewDimension::D2,
                )),
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();
    let error = device
        .create_bind_group(&BindGroupDesc::new(
            "incompatible-storage-bind-group",
            incompatible_layout,
            vec![BindGroupEntryDesc::new(
                3,
                BindGroupEntryResource::TextureView(view),
            )],
        ))
        .unwrap_err();

    assert_eq!(
        error,
        RhiError::InvalidBindGroupDescriptor {
            label: Some("incompatible-storage-bind-group".to_string()),
            reason: "binding 3 requires Rgba16Float storage texture format, got Rgba8Unorm"
                .to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_rejects_unsupported_storage_texture_layouts() {
    let device = DeterministicRhiContractDevice::new_headless();

    let error = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "srgb-storage-texture-layout",
            vec![BindGroupLayoutEntryDesc::new(
                3,
                BindingResourceType::StorageTexture(StorageTextureBindingDesc::write_only(
                    TextureFormat::Rgba8UnormSrgb,
                    TextureViewDimension::D2,
                )),
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap_err();

    assert_eq!(
        error,
        RhiError::InvalidBindGroupLayoutDescriptor {
            label: Some("srgb-storage-texture-layout".to_string()),
            reason: "binding 3 storage texture format Rgba8UnormSrgb is not supported by the MVP storage texture ABI"
                .to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_binds_declared_srgb_sampled_view_and_keeps_storage_linear() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "runtime-mipgen-texture",
                4,
                4,
                TextureFormat::Rgba8Unorm,
                TextureUsage::SAMPLED | TextureUsage::STORAGE,
            )
            .with_view_formats([TextureFormat::Rgba8UnormSrgb]),
        )
        .unwrap();
    let srgb_view = device
        .create_texture_view(
            &TextureViewDesc::new("runtime-mipgen-srgb-srv", texture, TextureViewDimension::D2)
                .with_format(TextureFormat::Rgba8UnormSrgb),
        )
        .unwrap();
    let linear_view = device
        .create_texture_view(&TextureViewDesc::new(
            "runtime-mipgen-linear-uav",
            texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let sampled_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "runtime-mipgen-srgb-sampled-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::SampledTexture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                vec![ShaderStage::Fragment],
            )],
        ))
        .unwrap();
    let storage_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "runtime-mipgen-linear-storage-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::StorageTexture(StorageTextureBindingDesc::write_only(
                    TextureFormat::Rgba8Unorm,
                    TextureViewDimension::D2,
                )),
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();

    device
        .create_bind_group(&BindGroupDesc::new(
            "runtime-mipgen-srgb-sampled-bind-group",
            sampled_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(srgb_view),
            )],
        ))
        .unwrap();
    device
        .create_bind_group(&BindGroupDesc::new(
            "runtime-mipgen-linear-storage-bind-group",
            storage_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(linear_view),
            )],
        ))
        .unwrap();

    let error = device
        .create_bind_group(&BindGroupDesc::new(
            "runtime-mipgen-srgb-storage-bind-group",
            storage_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(srgb_view),
            )],
        ))
        .unwrap_err();
    assert_eq!(
        error,
        RhiError::InvalidBindGroupDescriptor {
            label: Some("runtime-mipgen-srgb-storage-bind-group".to_string()),
            reason: "binding 0 requires Rgba8Unorm storage texture format, got Rgba8UnormSrgb"
                .to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_rejects_undeclared_texture_view_format() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(&TextureDesc::new(
            "undeclared-view-format-texture",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let view = TextureViewDesc::new("undeclared-srgb-view", texture, TextureViewDimension::D2)
        .with_format(TextureFormat::Rgba8UnormSrgb);

    assert_eq!(
        device.create_texture_view(&view).unwrap_err(),
        RhiError::InvalidTextureViewDescriptor {
            label: Some("undeclared-srgb-view".to_string()),
            reason: "view format Rgba8UnormSrgb was not declared by parent texture".to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_rejects_invalid_declared_texture_view_formats() {
    let device = DeterministicRhiContractDevice::new_headless();
    let repeated_parent_format = TextureDesc::new(
        "repeated-parent-view-format-texture",
        4,
        4,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED,
    )
    .with_view_formats([TextureFormat::Rgba8Unorm]);
    assert_eq!(
        device.create_texture(&repeated_parent_format).unwrap_err(),
        RhiError::InvalidTextureDescriptor {
            label: Some("repeated-parent-view-format-texture".to_string()),
            reason: "view format Rgba8Unorm repeats the parent texture format".to_string(),
        }
    );

    let duplicate_alternate_format = TextureDesc::new(
        "duplicate-alternate-view-format-texture",
        4,
        4,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED,
    )
    .with_view_formats([TextureFormat::Rgba8UnormSrgb, TextureFormat::Rgba8UnormSrgb]);
    assert_eq!(
        device
            .create_texture(&duplicate_alternate_format)
            .unwrap_err(),
        RhiError::InvalidTextureDescriptor {
            label: Some("duplicate-alternate-view-format-texture".to_string()),
            reason: "view format Rgba8UnormSrgb is declared more than once".to_string(),
        }
    );

    let incompatible_format = TextureDesc::new(
        "incompatible-view-format-texture",
        4,
        4,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED,
    )
    .with_view_formats([TextureFormat::Rgba16Float]);
    assert_eq!(
        device.create_texture(&incompatible_format).unwrap_err(),
        RhiError::InvalidTextureDescriptor {
            label: Some("incompatible-view-format-texture".to_string()),
            reason: "view format Rgba16Float cannot reinterpret parent texture format Rgba8Unorm"
                .to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_binds_depth_and_stencil_only_texture_views() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(&TextureDesc::new(
            "depth-stencil-source",
            4,
            4,
            TextureFormat::Depth24PlusStencil8,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        ))
        .unwrap();
    let depth_view = device
        .create_texture_view(
            &TextureViewDesc::new("depth-only-srv", texture, TextureViewDimension::D2)
                .with_aspect(TextureViewAspect::DepthOnly),
        )
        .unwrap();
    let stencil_view = device
        .create_texture_view(
            &TextureViewDesc::new("stencil-only-srv", texture, TextureViewDimension::D2)
                .with_aspect(TextureViewAspect::StencilOnly),
        )
        .unwrap();
    let combined_view = device
        .create_texture_view(&TextureViewDesc::new(
            "combined-depth-stencil-attachment",
            texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let depth_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "depth-only-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::SampledTexture {
                    sample_type: TextureSampleType::Depth,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();
    let stencil_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "stencil-only-layout",
            vec![BindGroupLayoutEntryDesc::new(
                0,
                BindingResourceType::SampledTexture {
                    sample_type: TextureSampleType::Uint,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                vec![ShaderStage::Compute],
            )],
        ))
        .unwrap();

    device
        .create_bind_group(&BindGroupDesc::new(
            "depth-only-bind-group",
            depth_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(depth_view),
            )],
        ))
        .unwrap();
    device
        .create_bind_group(&BindGroupDesc::new(
            "stencil-only-bind-group",
            stencil_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(stencil_view),
            )],
        ))
        .unwrap();
    assert_eq!(
        device
            .create_bind_group(&BindGroupDesc::new(
                "combined-depth-stencil-sampled-bind-group",
                depth_layout,
                vec![BindGroupEntryDesc::new(
                    0,
                    BindGroupEntryResource::TextureView(combined_view),
                )],
            ))
            .unwrap_err(),
        RhiError::InvalidBindGroupDescriptor {
            label: Some("combined-depth-stencil-sampled-bind-group".to_string()),
            reason: "binding 0 requires a shader-sampleable texture view aspect".to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_rejects_invalid_texture_view_aspects() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color_texture = device
        .create_texture(&TextureDesc::new(
            "color-aspect-source",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let color_depth_view = TextureViewDesc::new(
        "color-depth-only-view",
        color_texture,
        TextureViewDimension::D2,
    )
    .with_aspect(TextureViewAspect::DepthOnly);
    assert_eq!(
        device.create_texture_view(&color_depth_view).unwrap_err(),
        RhiError::InvalidTextureViewDescriptor {
            label: Some("color-depth-only-view".to_string()),
            reason: "depth-only aspect requires a depth texture".to_string(),
        }
    );

    let depth_texture = device
        .create_texture(&TextureDesc::new(
            "depth-aspect-source",
            4,
            4,
            TextureFormat::Depth32Float,
            TextureUsage::SAMPLED,
        ))
        .unwrap();
    let depth_stencil_view = TextureViewDesc::new(
        "depth-stencil-only-view",
        depth_texture,
        TextureViewDimension::D2,
    )
    .with_aspect(TextureViewAspect::StencilOnly);
    assert_eq!(
        device.create_texture_view(&depth_stencil_view).unwrap_err(),
        RhiError::InvalidTextureViewDescriptor {
            label: Some("depth-stencil-only-view".to_string()),
            reason: "stencil-only aspect requires a depth-stencil texture".to_string(),
        }
    );
}
