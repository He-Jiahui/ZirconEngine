use zr_rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupLayoutDesc,
    BindGroupLayoutEntryDesc, BindingResourceType, CommandList, PipelineDesc, PipelineKind,
    PipelineLayoutDesc, RenderDevice, RenderQueueClass, RhiError, SamplerBindingType, SamplerDesc,
    ShaderModuleDesc, ShaderStage, StorageTextureBindingDesc, TextureDesc, TextureFormat,
    TextureSampleType, TextureUsage, TextureViewAspect, TextureViewDesc, TextureViewDimension,
};

use super::{production_test_device, wait_for_submission};

#[test]
fn production_sampled_texture_view_encodes_without_exposing_native_handles() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "production-material-texture",
                4,
                4,
                TextureFormat::Rgba8UnormSrgb,
                TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            )
            .with_mip_levels(2),
        )
        .unwrap();
    let view_desc = TextureViewDesc::new(
        "production-material-view",
        texture,
        TextureViewDimension::D2,
    );
    let view = device.create_texture_view(&view_desc).unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear_mipmap_linear(
            "production-material-sampler",
        ))
        .unwrap();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "production-material-layout",
            vec![
                BindGroupLayoutEntryDesc::new(
                    0,
                    BindingResourceType::SampledTexture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
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
        ))
        .unwrap();
    let bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "production-material-bind-group",
            layout,
            vec![
                BindGroupEntryDesc::new(0, BindGroupEntryResource::TextureView(view)),
                BindGroupEntryDesc::new(1, BindGroupEntryResource::Sampler(sampler)),
            ],
        ))
        .unwrap();

    assert_eq!(device.texture_view_desc(view).unwrap(), view_desc);
    assert_eq!(
        device.destroy_texture(texture).unwrap_err(),
        RhiError::TextureHasLiveViews {
            texture: texture.diagnostic_id(),
            live_views: 1,
        }
    );

    device.destroy_bind_group(bind_group).unwrap();
    device.destroy_bind_group_layout(layout).unwrap();
    device.destroy_sampler(sampler).unwrap();
    device.destroy_texture_view(view).unwrap();
    device.destroy_texture(texture).unwrap();
}

#[test]
fn production_write_only_storage_texture_view_encodes_without_exposing_native_handles() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(&TextureDesc::new(
            "production-mip-generation-target",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        ))
        .unwrap();
    let view = device
        .create_texture_view(&TextureViewDesc::new(
            "production-mip-generation-target-uav",
            texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let layout_desc = BindGroupLayoutDesc::new(
        "production-mip-generation-layout",
        vec![BindGroupLayoutEntryDesc::new(
            0,
            BindingResourceType::StorageTexture(StorageTextureBindingDesc::write_only(
                TextureFormat::Rgba8Unorm,
                TextureViewDimension::D2,
            )),
            vec![ShaderStage::Compute],
        )],
    );
    let layout = device.create_bind_group_layout(&layout_desc).unwrap();
    let bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "production-mip-generation-bind-group",
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(view),
            )],
        ))
        .unwrap();

    assert_eq!(device.bind_group_layout_desc(layout).unwrap(), layout_desc);
    device.destroy_bind_group(bind_group).unwrap();
    device.destroy_bind_group_layout(layout).unwrap();
    device.destroy_texture_view(view).unwrap();
    device.destroy_texture(texture).unwrap();
}

#[test]
fn production_write_only_storage_texture_view_dispatches_through_the_neutral_submission_path() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(&TextureDesc::new(
            "production-storage-dispatch-target",
            1,
            1,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        ))
        .unwrap();
    let view = device
        .create_texture_view(&TextureViewDesc::new(
            "production-storage-dispatch-target-uav",
            texture,
            TextureViewDimension::D2,
        ))
        .unwrap();
    let bind_group_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "production-storage-dispatch-layout",
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
    let bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "production-storage-dispatch-bind-group",
            bind_group_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(view),
            )],
        ))
        .unwrap();
    let pipeline_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "production-storage-dispatch-pipeline-layout",
            vec![bind_group_layout],
        ))
        .unwrap();
    let shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "production-storage-dispatch-shader",
            ShaderStage::Compute,
            "main",
            "@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;\n@compute @workgroup_size(1) fn main() { textureStore(output_texture, vec2<i32>(0, 0), vec4<f32>(0.0, 0.5, 1.0, 1.0)); }",
        ))
        .unwrap();
    let pipeline = device
        .create_pipeline(
            &PipelineDesc::new(
                "production-storage-dispatch-pipeline",
                PipelineKind::Compute,
            )
            .with_layout(pipeline_layout)
            .with_compute_shader(shader),
        )
        .unwrap_or_else(|error| {
            panic!(
                "storage-texture compute pipeline creation failed: {error:?}; first fault: {:?}",
                device.first_fault()
            )
        });
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "production-storage-dispatch")
        .unwrap();
    command_list.begin_compute_pass("production-storage-dispatch-pass");
    command_list.set_pipeline(pipeline);
    command_list.set_bind_group(0, bind_group);
    command_list.dispatch_compute(1, 1, 1);
    command_list.end_compute_pass();
    wait_for_submission(&device, device.submit(command_list).unwrap());

    device.destroy_pipeline(pipeline).unwrap();
    device.destroy_shader_module(shader).unwrap();
    device.destroy_pipeline_layout(pipeline_layout).unwrap();
    device.destroy_bind_group(bind_group).unwrap();
    device.destroy_bind_group_layout(bind_group_layout).unwrap();
    device.destroy_texture_view(view).unwrap();
    device.destroy_texture(texture).unwrap();
}

#[test]
fn production_declared_srgb_texture_view_creates_and_binds_natively() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(
            &TextureDesc::new(
                "production-runtime-mipgen-texture",
                4,
                4,
                TextureFormat::Rgba8Unorm,
                TextureUsage::SAMPLED,
            )
            .with_view_formats([TextureFormat::Rgba8UnormSrgb]),
        )
        .unwrap();
    let view_desc = TextureViewDesc::new(
        "production-runtime-mipgen-srgb-view",
        texture,
        TextureViewDimension::D2,
    )
    .with_format(TextureFormat::Rgba8UnormSrgb);
    let view = device.create_texture_view(&view_desc).unwrap();
    let layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "production-runtime-mipgen-srgb-layout",
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
    let bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "production-runtime-mipgen-srgb-bind-group",
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(view),
            )],
        ))
        .unwrap();

    assert_eq!(device.texture_view_desc(view).unwrap(), view_desc);
    device.destroy_bind_group(bind_group).unwrap();
    device.destroy_bind_group_layout(layout).unwrap();
    device.destroy_texture_view(view).unwrap();
    device.destroy_texture(texture).unwrap();
}

#[test]
fn production_depth_and_stencil_only_texture_views_create_native_bind_groups() {
    let Some(device) = production_test_device() else {
        return;
    };
    let texture = device
        .create_texture(&TextureDesc::new(
            "production-depth-stencil-source",
            4,
            4,
            TextureFormat::Depth24PlusStencil8,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        ))
        .unwrap();
    let depth_view = device
        .create_texture_view(
            &TextureViewDesc::new(
                "production-depth-only-view",
                texture,
                TextureViewDimension::D2,
            )
            .with_aspect(TextureViewAspect::DepthOnly),
        )
        .unwrap();
    let stencil_view = device
        .create_texture_view(
            &TextureViewDesc::new(
                "production-stencil-only-view",
                texture,
                TextureViewDimension::D2,
            )
            .with_aspect(TextureViewAspect::StencilOnly),
        )
        .unwrap();
    let depth_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "production-depth-only-layout",
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
            "production-stencil-only-layout",
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
    let depth_bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "production-depth-only-bind-group",
            depth_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(depth_view),
            )],
        ))
        .unwrap();
    let stencil_bind_group = device
        .create_bind_group(&BindGroupDesc::new(
            "production-stencil-only-bind-group",
            stencil_layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::TextureView(stencil_view),
            )],
        ))
        .unwrap();

    device.destroy_bind_group(depth_bind_group).unwrap();
    device.destroy_bind_group(stencil_bind_group).unwrap();
    device.destroy_bind_group_layout(depth_layout).unwrap();
    device.destroy_bind_group_layout(stencil_layout).unwrap();
    device.destroy_texture_view(depth_view).unwrap();
    device.destroy_texture_view(stencil_view).unwrap();
    device.destroy_texture(texture).unwrap();
}
