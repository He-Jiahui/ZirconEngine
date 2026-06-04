use crate::rhi::ShaderModuleHandle;
use crate::rhi::{
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingResourceType, BlendFactor,
    BlendOperation, BlendStateDesc, BufferDesc, BufferUsage, ColorTargetDesc, ColorWriteMask,
    CompareFunction, CullMode, DepthStencilStateDesc, FilterMode, FrontFace, MipmapFilterMode,
    PipelineDesc, PipelineKind, PrimitiveStateDesc, PrimitiveTopology, RasterPipelineStateDesc,
    SamplerDesc, ShaderModuleDesc, ShaderStage, TextureDesc, TextureDimension, TextureFormat,
    TextureResidency, TextureUsage, VertexAttributeDesc, VertexBufferLayoutDesc, VertexFormat,
    VertexInputLayoutDesc, VertexStepMode,
};
use crate::rhi::{BindGroupLayoutHandle, PipelineLayoutDesc, PipelineLayoutHandle};

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
    assert!(BufferUsage::ALL.contains(BufferUsage::UNIFORM | BufferUsage::COPY_DST));
    assert!(!buffer.usage.has_unknown_bits());
    assert_eq!(texture.label.as_deref(), Some("scene-color"));
    assert_eq!(texture.width, 1920);
    assert_eq!(texture.height, 1080);
    assert_eq!(texture.dimension, TextureDimension::D2);
    assert_eq!(sampler.mag_filter, FilterMode::Linear);
    assert_eq!(sampler.min_filter, FilterMode::Linear);
    assert_eq!(sampler.mipmap_filter, MipmapFilterMode::Nearest);
    assert_eq!(pipeline.kind, PipelineKind::Raster);
}

#[test]
fn pipeline_descriptors_bind_shader_stages_to_pipeline_layouts() {
    let material_layout = BindGroupLayoutHandle::new(42);
    let pipeline_layout = PipelineLayoutDesc::new("forward-layout", vec![material_layout]);
    let vertex_shader = ShaderModuleDesc::new(
        "mesh-vs",
        ShaderStage::Vertex,
        "vs_main",
        "@vertex fn vs_main() {}",
    );
    let fragment_shader = ShaderModuleDesc::new(
        "mesh-fs",
        ShaderStage::Fragment,
        "fs_main",
        "@fragment fn fs_main() {}",
    );
    let raster_pipeline = PipelineDesc::new("forward-opaque", PipelineKind::Raster)
        .with_layout(PipelineLayoutHandle::new(9))
        .with_vertex_shader(ShaderModuleHandle::new(10))
        .with_fragment_shader(ShaderModuleHandle::new(11));
    let compute_pipeline = PipelineDesc::new("postprocess-blur", PipelineKind::Compute)
        .with_layout(PipelineLayoutHandle::new(12))
        .with_compute_shader(ShaderModuleHandle::new(13));

    assert_eq!(
        pipeline_layout.bind_group_layouts,
        vec![BindGroupLayoutHandle::new(42)]
    );
    assert_eq!(vertex_shader.stage, ShaderStage::Vertex);
    assert_eq!(fragment_shader.entry_point, "fs_main");
    assert_eq!(raster_pipeline.layout, Some(PipelineLayoutHandle::new(9)));
    assert_eq!(
        raster_pipeline.vertex_shader,
        Some(ShaderModuleHandle::new(10))
    );
    assert_eq!(
        raster_pipeline.fragment_shader,
        Some(ShaderModuleHandle::new(11))
    );
    assert_eq!(
        compute_pipeline.compute_shader,
        Some(ShaderModuleHandle::new(13))
    );
}

#[test]
fn raster_pipeline_state_descriptors_cover_scene_ui_postprocess_and_depth_targets() {
    let position_color_uv_layout = VertexInputLayoutDesc::new(vec![VertexBufferLayoutDesc::new(
        32,
        vec![
            VertexAttributeDesc::new(0, 0, VertexFormat::Float32x3),
            VertexAttributeDesc::new(1, 12, VertexFormat::Float32x4),
            VertexAttributeDesc::new(2, 28, VertexFormat::Float16x2),
        ],
    )]);
    let instance_layout = VertexBufferLayoutDesc::new(
        64,
        vec![
            VertexAttributeDesc::new(5, 0, VertexFormat::Float32x4),
            VertexAttributeDesc::new(6, 16, VertexFormat::Float32x4),
            VertexAttributeDesc::new(7, 32, VertexFormat::Float32x4),
            VertexAttributeDesc::new(8, 48, VertexFormat::Float32x4),
        ],
    )
    .with_step_mode(VertexStepMode::Instance);
    let scene_state = RasterPipelineStateDesc::single_color(TextureFormat::Rgba16Float)
        .with_depth_stencil(DepthStencilStateDesc::new(
            TextureFormat::Depth24PlusStencil8,
            true,
            CompareFunction::LessEqual,
        ))
        .with_primitive(
            PrimitiveStateDesc::triangle_list()
                .with_front_face(FrontFace::Ccw)
                .with_cull_mode(CullMode::Back),
        )
        .with_sample_count(4)
        .with_vertex_input(VertexInputLayoutDesc::new(vec![
            position_color_uv_layout.buffers[0].clone(),
            instance_layout,
        ]));
    let ui_state =
        RasterPipelineStateDesc::new(vec![ColorTargetDesc::new(TextureFormat::Bgra8UnormSrgb)
            .with_blend(BlendStateDesc::alpha_blending())])
        .with_primitive(PrimitiveStateDesc::triangle_list().with_cull_mode(CullMode::None))
        .with_vertex_input(position_color_uv_layout);
    let postprocess_state = RasterPipelineStateDesc::single_color(TextureFormat::Rgba16Float);
    let depth_prepass_state = RasterPipelineStateDesc::depth_only(DepthStencilStateDesc::new(
        TextureFormat::Depth32Float,
        true,
        CompareFunction::LessEqual,
    ));
    let alpha_only_target =
        ColorTargetDesc::new(TextureFormat::Rgba8Unorm).with_write_mask(ColorWriteMask::ALPHA);
    let line_state = RasterPipelineStateDesc::new(vec![alpha_only_target]).with_primitive(
        PrimitiveStateDesc::default()
            .with_topology(PrimitiveTopology::LineList)
            .with_front_face(FrontFace::Cw),
    );
    let pipeline = PipelineDesc::new("forward-opaque", PipelineKind::Raster)
        .with_layout(PipelineLayoutHandle::new(9))
        .with_vertex_shader(ShaderModuleHandle::new(10))
        .with_fragment_shader(ShaderModuleHandle::new(11))
        .with_raster_state(scene_state.clone());

    assert_eq!(
        scene_state.color_targets[0].format,
        TextureFormat::Rgba16Float
    );
    assert_eq!(scene_state.sample_count, 4);
    assert_eq!(scene_state.vertex_input.buffers.len(), 2);
    assert_eq!(
        scene_state.depth_stencil.unwrap().format,
        TextureFormat::Depth24PlusStencil8
    );
    assert_eq!(scene_state.primitive.cull_mode, CullMode::Back);
    assert_eq!(ui_state.primitive.cull_mode, CullMode::None);
    assert_eq!(
        ui_state.color_targets[0].blend.unwrap().color.src_factor,
        BlendFactor::SrcAlpha
    );
    assert_eq!(
        ui_state.color_targets[0].blend.unwrap().alpha.operation,
        BlendOperation::Add
    );
    assert_eq!(postprocess_state.color_targets.len(), 1);
    assert!(postprocess_state.color_targets[0].blend.is_none());
    assert!(depth_prepass_state.color_targets.is_empty());
    assert!(depth_prepass_state.depth_stencil.is_some());
    assert!(depth_prepass_state.vertex_input.buffers.is_empty());
    assert_eq!(line_state.primitive.topology, PrimitiveTopology::LineList);
    assert_eq!(line_state.primitive.front_face, FrontFace::Cw);
    assert_eq!(
        line_state.color_targets[0].write_mask.bits(),
        ColorWriteMask::ALPHA.bits()
    );
    assert!(!line_state.color_targets[0].write_mask.has_unknown_bits());
    assert_eq!(VertexFormat::Float32x3.size_bytes(), 12);
    assert_eq!(VertexFormat::Unorm8x4.size_bytes(), 4);
    assert!(ColorWriteMask::ALL.contains(ColorWriteMask::COLOR));
    assert_eq!(pipeline.raster_state, Some(scene_state));
}

#[test]
fn sampler_descriptors_cover_mips_lod_compare_and_anisotropy() {
    let material_sampler = SamplerDesc::linear_mipmap_linear("material-trilinear")
        .with_lod_clamp(0.0, 8.0)
        .with_anisotropy_clamp(8);

    assert_eq!(material_sampler.mag_filter, FilterMode::Linear);
    assert_eq!(material_sampler.min_filter, FilterMode::Linear);
    assert_eq!(material_sampler.mipmap_filter, MipmapFilterMode::Linear);
    assert_eq!(material_sampler.lod_min_clamp, 0.0);
    assert_eq!(material_sampler.lod_max_clamp, 8.0);
    assert_eq!(material_sampler.anisotropy_clamp, 8);
    assert_eq!(material_sampler.compare, None);

    let shadow_sampler = SamplerDesc::nearest("shadow-compare")
        .with_compare(CompareFunction::LessEqual)
        .with_lod_clamp(0.0, 0.0);

    assert_eq!(shadow_sampler.mag_filter, FilterMode::Nearest);
    assert_eq!(shadow_sampler.min_filter, FilterMode::Nearest);
    assert_eq!(shadow_sampler.compare, Some(CompareFunction::LessEqual));
    assert!(!shadow_sampler.uses_anisotropy());
}

#[test]
fn bind_group_layout_descriptors_cover_material_texture_and_sampler_bindings() {
    let layout = BindGroupLayoutDesc::new(
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
    );

    assert_eq!(layout.label.as_deref(), Some("material-bindings"));
    assert_eq!(layout.entries.len(), 3);
    assert_eq!(layout.entries[0].binding, 0);
    assert_eq!(
        layout.entries[0].resource_type,
        BindingResourceType::UniformBuffer
    );
    assert_eq!(
        layout.entries[0].visibility,
        vec![ShaderStage::Vertex, ShaderStage::Fragment]
    );
    assert_eq!(
        layout.entries[1].resource_type,
        BindingResourceType::Texture
    );
    assert_eq!(
        layout.entries[2].resource_type,
        BindingResourceType::Sampler
    );
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
fn texture_descriptors_report_mip_capacity_for_shape_validation() {
    let d2 = TextureDesc::new(
        "material-mips",
        256,
        128,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::SAMPLED,
    );
    let d3 = TextureDesc::new(
        "volume-mips",
        32,
        16,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED,
    )
    .with_dimension(TextureDimension::D3)
    .with_depth(8);

    assert_eq!(d2.max_full_mip_levels(), 9);
    assert_eq!(d3.max_full_mip_levels(), 6);
    assert!(d2.mip_levels_fit_shape());
    assert!(!d2.with_mip_levels(10).mip_levels_fit_shape());
}

#[test]
fn rhi_descriptors_do_not_embed_scene_level_semantics() {
    let source = format!(
        "{}\n{}",
        include_str!("../descriptors.rs"),
        include_str!("../descriptors/pipeline.rs")
    );
    for forbidden in ["Mesh", "Material", "Light", "Scene"] {
        assert!(
            !source.contains(forbidden),
            "RHI descriptors must not encode upper-layer `{forbidden}` semantics"
        );
    }
}
