use crate::rhi::{
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindGroupLayoutHandle, BindingResourceType,
    BlendStateDesc, ColorTargetDesc, ColorWriteMask, CompareFunction, DepthStencilStateDesc,
    PipelineDesc, PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle, RasterPipelineStateDesc,
    RenderDevice, RhiError, ShaderModuleDesc, ShaderModuleHandle, ShaderStage, TextureFormat,
    VertexAttributeDesc, VertexBufferLayoutDesc, VertexFormat, VertexInputLayoutDesc,
    VertexStepMode,
};
use crate::rhi_wgpu::DeterministicRhiContractDevice;

fn create_pipeline_layout(device: &DeterministicRhiContractDevice) -> PipelineLayoutHandle {
    device
        .create_pipeline_layout(&PipelineLayoutDesc::new("pipeline-layout", Vec::new()))
        .unwrap()
}

fn create_test_bind_group_layout_desc(label: &str) -> BindGroupLayoutDesc {
    BindGroupLayoutDesc::new(
        label,
        vec![BindGroupLayoutEntryDesc::new(
            0,
            BindingResourceType::UniformBuffer,
            vec![ShaderStage::Vertex, ShaderStage::Fragment],
        )],
    )
}

fn create_shader(
    device: &DeterministicRhiContractDevice,
    label: &str,
    stage: ShaderStage,
    entry_point: &str,
    source: &str,
) -> ShaderModuleHandle {
    device
        .create_shader_module(&ShaderModuleDesc::new(label, stage, entry_point, source))
        .unwrap()
}

fn create_vertex_shader(device: &DeterministicRhiContractDevice) -> ShaderModuleHandle {
    create_shader(
        device,
        "vs",
        ShaderStage::Vertex,
        "vs_main",
        "@vertex fn vs_main() {}",
    )
}

fn create_fragment_shader(device: &DeterministicRhiContractDevice) -> ShaderModuleHandle {
    create_shader(
        device,
        "fs",
        ShaderStage::Fragment,
        "fs_main",
        "@fragment fn fs_main() {}",
    )
}

fn create_compute_shader(device: &DeterministicRhiContractDevice) -> ShaderModuleHandle {
    create_shader(
        device,
        "cs",
        ShaderStage::Compute,
        "main",
        "@compute @workgroup_size(1) fn main() {}",
    )
}

fn create_color_raster_desc(
    label: &str,
    layout: PipelineLayoutHandle,
    vertex: ShaderModuleHandle,
    fragment: ShaderModuleHandle,
    raster_state: RasterPipelineStateDesc,
) -> PipelineDesc {
    PipelineDesc::new(label, PipelineKind::Raster)
        .with_layout(layout)
        .with_vertex_shader(vertex)
        .with_fragment_shader(fragment)
        .with_raster_state(raster_state)
}

fn create_vertex_input_layout() -> VertexInputLayoutDesc {
    VertexInputLayoutDesc::new(vec![
        VertexBufferLayoutDesc::new(
            32,
            vec![
                VertexAttributeDesc::new(0, 0, VertexFormat::Float32x3),
                VertexAttributeDesc::new(1, 12, VertexFormat::Float32x3),
                VertexAttributeDesc::new(2, 24, VertexFormat::Float32x2),
            ],
        ),
        VertexBufferLayoutDesc::new(
            64,
            vec![
                VertexAttributeDesc::new(5, 0, VertexFormat::Float32x4),
                VertexAttributeDesc::new(6, 16, VertexFormat::Float32x4),
                VertexAttributeDesc::new(7, 32, VertexFormat::Float32x4),
                VertexAttributeDesc::new(8, 48, VertexFormat::Float32x4),
            ],
        )
        .with_step_mode(VertexStepMode::Instance),
    ])
}

#[test]
fn deterministic_rhi_contract_roundtrips_pipeline_layouts_and_shader_bound_pipelines() {
    let device = DeterministicRhiContractDevice::new_headless();
    let material_layout = device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
            "material-bindings",
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
    let pipeline_layout_desc = PipelineLayoutDesc::new("forward-layout", vec![material_layout]);
    let pipeline_layout = device
        .create_pipeline_layout(&pipeline_layout_desc)
        .unwrap();
    let vertex = create_vertex_shader(&device);
    let fragment = create_fragment_shader(&device);
    let pipeline_desc = create_color_raster_desc(
        "forward-opaque",
        pipeline_layout,
        vertex,
        fragment,
        RasterPipelineStateDesc::single_color(TextureFormat::Rgba8UnormSrgb),
    );

    let pipeline = device.create_pipeline(&pipeline_desc).unwrap();

    assert_eq!(
        device.pipeline_layout_desc(pipeline_layout).unwrap(),
        pipeline_layout_desc
    );
    assert_eq!(device.pipeline_desc(pipeline).unwrap(), pipeline_desc);

    device.destroy_pipeline(pipeline).unwrap();
    assert_eq!(
        device.pipeline_desc(pipeline).unwrap_err(),
        RhiError::UnknownPipeline(pipeline.raw())
    );
    device.destroy_pipeline_layout(pipeline_layout).unwrap();
    assert_eq!(
        device.pipeline_layout_desc(pipeline_layout).unwrap_err(),
        RhiError::UnknownPipelineLayout(pipeline_layout.raw())
    );
}

#[test]
fn deterministic_rhi_contract_rejects_invalid_shader_and_pipeline_descriptors() {
    let device = DeterministicRhiContractDevice::new_headless();

    assert_eq!(
        device
            .create_shader_module(&ShaderModuleDesc::new(
                "empty-shader",
                ShaderStage::Compute,
                "main",
                ""
            ))
            .unwrap_err(),
        RhiError::InvalidShaderModuleDescriptor {
            label: Some("empty-shader".to_string()),
            reason: "shader source must not be empty".to_string(),
        }
    );
    assert_eq!(
        device
            .create_shader_module(&ShaderModuleDesc::new(
                "empty-entry",
                ShaderStage::Compute,
                "",
                "@compute @workgroup_size(1) fn main() {}",
            ))
            .unwrap_err(),
        RhiError::InvalidShaderModuleDescriptor {
            label: Some("empty-entry".to_string()),
            reason: "shader entry point must not be empty".to_string(),
        }
    );

    let empty_pipeline_layout_desc = PipelineLayoutDesc::new("empty-pipeline-layout", Vec::new());
    let empty_pipeline_layout = device
        .create_pipeline_layout(&empty_pipeline_layout_desc)
        .unwrap();
    assert_eq!(
        device.pipeline_layout_desc(empty_pipeline_layout).unwrap(),
        empty_pipeline_layout_desc
    );
    assert_eq!(
        device
            .create_pipeline_layout(&PipelineLayoutDesc::new(
                "unknown-layout",
                vec![BindGroupLayoutHandle::new(999)],
            ))
            .unwrap_err(),
        RhiError::UnknownBindGroupLayout(999)
    );

    let bind_group_layout = device
        .create_bind_group_layout(&create_test_bind_group_layout_desc("pipeline-bind-group"))
        .unwrap();
    assert_eq!(
        device
            .create_pipeline_layout(&PipelineLayoutDesc::new(
                "duplicate-layout",
                vec![bind_group_layout, bind_group_layout],
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineLayoutDescriptor {
            label: Some("duplicate-layout".to_string()),
            reason: format!("duplicate bind group layout `{}`", bind_group_layout.raw()),
        }
    );

    let pipeline_layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            "valid-layout",
            vec![bind_group_layout],
        ))
        .unwrap();
    let compute = create_compute_shader(&device);
    let vertex = create_vertex_shader(&device);
    let fragment = create_fragment_shader(&device);

    assert_eq!(
        device
            .create_pipeline(&PipelineDesc::new("layout-missing", PipelineKind::Compute))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("layout-missing".to_string()),
            reason: "pipeline must reference a pipeline layout".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("compute-missing-shader", PipelineKind::Compute)
                    .with_layout(pipeline_layout),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("compute-missing-shader".to_string()),
            reason: "compute pipeline requires a compute shader".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("compute-wrong-shader", PipelineKind::Compute)
                    .with_layout(pipeline_layout)
                    .with_compute_shader(vertex),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("compute-wrong-shader".to_string()),
            reason: format!(
                "shader `{}` stage {:?} does not match required stage {:?}",
                vertex.raw(),
                ShaderStage::Vertex,
                ShaderStage::Compute
            ),
        }
    );
    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("raster-missing-fragment", PipelineKind::Raster)
                    .with_layout(pipeline_layout)
                    .with_vertex_shader(vertex),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-missing-fragment".to_string()),
            reason: "raster pipeline requires a fragment shader".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("raster-with-compute", PipelineKind::Raster)
                    .with_layout(pipeline_layout)
                    .with_vertex_shader(vertex)
                    .with_fragment_shader(fragment)
                    .with_compute_shader(compute)
                    .with_raster_state(RasterPipelineStateDesc::single_color(
                        TextureFormat::Rgba8UnormSrgb,
                    )),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-with-compute".to_string()),
            reason: "raster pipeline must not reference a compute shader".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("ray-tracing", PipelineKind::RayTracing)
                    .with_layout(pipeline_layout),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("ray-tracing".to_string()),
            reason: "ray tracing pipelines are not supported by the WGPU backend contract yet"
                .to_string(),
        }
    );
}

#[test]
fn deterministic_rhi_contract_roundtrips_raster_pipeline_state_for_color_depth_and_depth_only() {
    let device = DeterministicRhiContractDevice::new_headless();
    let layout = create_pipeline_layout(&device);
    let vertex = create_vertex_shader(&device);
    let fragment = create_fragment_shader(&device);

    let scene_state =
        RasterPipelineStateDesc::new(vec![ColorTargetDesc::new(TextureFormat::Rgba16Float)
            .with_blend(BlendStateDesc::alpha_blending())])
        .with_depth_stencil(
            DepthStencilStateDesc::new(
                TextureFormat::Depth24PlusStencil8,
                true,
                CompareFunction::LessEqual,
            )
            .with_stencil_enabled(true),
        )
        .with_sample_count(4)
        .with_vertex_input(create_vertex_input_layout());
    let scene_pipeline = create_color_raster_desc(
        "forward-scene",
        layout,
        vertex,
        fragment,
        scene_state.clone(),
    );
    let scene_handle = device.create_pipeline(&scene_pipeline).unwrap();

    let depth_state = RasterPipelineStateDesc::depth_only(DepthStencilStateDesc::new(
        TextureFormat::Depth32Float,
        true,
        CompareFunction::LessEqual,
    ));
    let depth_pipeline = PipelineDesc::new("shadow-depth", PipelineKind::Raster)
        .with_layout(layout)
        .with_vertex_shader(vertex)
        .with_raster_state(depth_state.clone());
    let depth_handle = device.create_pipeline(&depth_pipeline).unwrap();

    assert_eq!(device.pipeline_desc(scene_handle).unwrap(), scene_pipeline);
    assert_eq!(device.pipeline_desc(depth_handle).unwrap(), depth_pipeline);
    assert_eq!(
        device
            .pipeline_desc(scene_handle)
            .unwrap()
            .raster_state
            .unwrap(),
        scene_state
    );
    let stored_scene_state = device
        .pipeline_desc(scene_handle)
        .unwrap()
        .raster_state
        .unwrap();
    assert!(stored_scene_state.color_targets[0].blend.is_some());
    assert_eq!(stored_scene_state.vertex_input.buffers.len(), 2);
    assert_eq!(
        stored_scene_state.vertex_input.buffers[1].step_mode,
        VertexStepMode::Instance
    );
    assert_eq!(
        device
            .pipeline_desc(depth_handle)
            .unwrap()
            .raster_state
            .unwrap(),
        depth_state
    );
}

#[test]
fn deterministic_rhi_contract_rejects_invalid_raster_pipeline_state_descriptors() {
    let device = DeterministicRhiContractDevice::new_headless();
    let layout = create_pipeline_layout(&device);
    let vertex = create_vertex_shader(&device);
    let fragment = create_fragment_shader(&device);
    let compute = create_compute_shader(&device);

    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("raster-missing-state", PipelineKind::Raster)
                    .with_layout(layout)
                    .with_vertex_shader(vertex)
                    .with_fragment_shader(fragment),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-missing-state".to_string()),
            reason: "raster pipeline requires raster state".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-empty-targets",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::new(Vec::new()),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-empty-targets".to_string()),
            reason: "raster pipeline requires at least one color target or depth/stencil target"
                .to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-zero-samples",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm)
                    .with_sample_count(0),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-zero-samples".to_string()),
            reason: "raster pipeline sample_count must be greater than zero".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-depth-color-target",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::new(vec![ColorTargetDesc::new(
                    TextureFormat::Depth24Plus,
                )]),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-depth-color-target".to_string()),
            reason: "color target 0 must use a color format".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-empty-write-mask",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::new(vec![
                    ColorTargetDesc::new(TextureFormat::Rgba8Unorm,)
                        .with_write_mask(ColorWriteMask::NONE)
                ]),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-empty-write-mask".to_string()),
            reason: "color target 0 write mask must not be empty".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-empty-vertex-buffer",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm).with_vertex_input(
                    VertexInputLayoutDesc::new(vec![VertexBufferLayoutDesc::new(16, Vec::new())])
                ),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-empty-vertex-buffer".to_string()),
            reason: "vertex buffer layout 0 must declare attributes".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-zero-vertex-stride",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm).with_vertex_input(
                    VertexInputLayoutDesc::new(vec![VertexBufferLayoutDesc::new(
                        0,
                        vec![VertexAttributeDesc::new(0, 0, VertexFormat::Float32)]
                    )])
                ),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-zero-vertex-stride".to_string()),
            reason: "vertex buffer layout 0 stride must be greater than zero".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-duplicate-vertex-location",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm).with_vertex_input(
                    VertexInputLayoutDesc::new(vec![VertexBufferLayoutDesc::new(
                        24,
                        vec![
                            VertexAttributeDesc::new(0, 0, VertexFormat::Float32x3),
                            VertexAttributeDesc::new(0, 12, VertexFormat::Float32x3),
                        ],
                    )])
                ),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-duplicate-vertex-location".to_string()),
            reason: "vertex attribute shader location 0 is declared more than once".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-vertex-attribute-over-stride",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm).with_vertex_input(
                    VertexInputLayoutDesc::new(vec![VertexBufferLayoutDesc::new(
                        12,
                        vec![VertexAttributeDesc::new(0, 8, VertexFormat::Float32x3)]
                    )])
                ),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-vertex-attribute-over-stride".to_string()),
            reason: "vertex attribute 0 in buffer 0 exceeds array stride".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-color-depth-target",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm)
                    .with_depth_stencil(DepthStencilStateDesc::new(
                        TextureFormat::Rgba8Unorm,
                        true,
                        CompareFunction::LessEqual,
                    )),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-color-depth-target".to_string()),
            reason: "depth/stencil target must use a depth format".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(&create_color_raster_desc(
                "raster-stencil-with-depth-only-format",
                layout,
                vertex,
                fragment,
                RasterPipelineStateDesc::single_color(TextureFormat::Rgba8Unorm)
                    .with_depth_stencil(
                        DepthStencilStateDesc::new(
                            TextureFormat::Depth32Float,
                            true,
                            CompareFunction::LessEqual,
                        )
                        .with_stencil_enabled(true),
                    ),
            ))
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("raster-stencil-with-depth-only-format".to_string()),
            reason: "stencil state requires a stencil-capable depth format".to_string(),
        }
    );
    assert_eq!(
        device
            .create_pipeline(
                &PipelineDesc::new("compute-with-raster-state", PipelineKind::Compute)
                    .with_layout(layout)
                    .with_compute_shader(compute)
                    .with_raster_state(RasterPipelineStateDesc::single_color(
                        TextureFormat::Rgba8Unorm,
                    )),
            )
            .unwrap_err(),
        RhiError::InvalidPipelineDescriptor {
            label: Some("compute-with-raster-state".to_string()),
            reason: "compute pipeline must not declare raster state".to_string(),
        }
    );
}
