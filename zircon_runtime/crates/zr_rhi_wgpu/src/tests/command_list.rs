use crate::{DeterministicRhiContractCommandList, DeterministicRhiContractDevice};
use zr_rhi::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupHandle,
    BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindGroupLayoutHandle, BindingResourceType,
    BufferDesc, BufferHandle, BufferUsage, CommandList, CompareFunction, DepthStencilStateDesc,
    DiagnosticQueryPlan, DiagnosticQueryPlanError, DiagnosticReadbackBudget, IndexFormat,
    PipelineDesc, PipelineHandle, PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle,
    RasterPipelineStateDesc, RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc,
    RenderPassColorLoadOp, RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc,
    RenderPassStoreOp, RenderQueueClass, RhiError, ShaderModuleDesc, ShaderModuleHandle,
    ShaderStage, TextureDesc, TextureFormat, TextureHandle, TextureUsage, VertexAttributeDesc,
    VertexBufferLayoutDesc, VertexFormat, VertexInputLayoutDesc, VertexStepMode,
};

fn create_compute_pipeline(
    device: &DeterministicRhiContractDevice,
    label: &str,
    shader: ShaderModuleHandle,
) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            format!("{label}-layout"),
            Vec::new(),
        ))
        .unwrap();
    create_compute_pipeline_with_layout(device, label, shader, layout)
}

fn create_compute_pipeline_with_layout(
    device: &DeterministicRhiContractDevice,
    label: &str,
    shader: ShaderModuleHandle,
    layout: PipelineLayoutHandle,
) -> PipelineHandle {
    device
        .create_pipeline(
            &PipelineDesc::new(label, PipelineKind::Compute)
                .with_layout(layout)
                .with_compute_shader(shader),
        )
        .unwrap()
}

fn create_raster_pipeline(
    device: &DeterministicRhiContractDevice,
    label: &str,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
) -> PipelineHandle {
    create_raster_pipeline_with_vertex_input(
        device,
        label,
        vertex_shader,
        fragment_shader,
        VertexInputLayoutDesc::empty(),
    )
}

fn create_raster_pipeline_with_vertex_input(
    device: &DeterministicRhiContractDevice,
    label: &str,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
    vertex_input: VertexInputLayoutDesc,
) -> PipelineHandle {
    let layout = device
        .create_pipeline_layout(&PipelineLayoutDesc::new(
            format!("{label}-layout"),
            Vec::new(),
        ))
        .unwrap();
    create_raster_pipeline_with_layout_and_vertex_input(
        device,
        label,
        vertex_shader,
        fragment_shader,
        layout,
        vertex_input,
    )
}

fn create_raster_pipeline_with_layout_and_vertex_input(
    device: &DeterministicRhiContractDevice,
    label: &str,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
    layout: PipelineLayoutHandle,
    vertex_input: VertexInputLayoutDesc,
) -> PipelineHandle {
    device
        .create_pipeline(
            &PipelineDesc::new(label, PipelineKind::Raster)
                .with_layout(layout)
                .with_vertex_shader(vertex_shader)
                .with_fragment_shader(fragment_shader)
                .with_raster_state(
                    RasterPipelineStateDesc::single_color(TextureFormat::Rgba8UnormSrgb)
                        .with_depth_stencil(DepthStencilStateDesc::new(
                            TextureFormat::Depth24Plus,
                            true,
                            CompareFunction::LessEqual,
                        ))
                        .with_vertex_input(vertex_input),
                ),
        )
        .unwrap()
}

fn create_uniform_bind_group_layout(
    device: &DeterministicRhiContractDevice,
    label: &str,
) -> BindGroupLayoutHandle {
    device
        .create_bind_group_layout(&BindGroupLayoutDesc::new(
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
        ))
        .unwrap()
}

fn create_uniform_bind_group(
    device: &DeterministicRhiContractDevice,
    label: &str,
    layout: BindGroupLayoutHandle,
) -> BindGroupHandle {
    let buffer = device
        .create_buffer(&BufferDesc::new(
            format!("{label}-uniform"),
            64,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    device
        .create_bind_group(&BindGroupDesc::new(
            label,
            layout,
            vec![BindGroupEntryDesc::new(
                0,
                BindGroupEntryResource::Buffer(zr_rhi::BindGroupBufferBinding::whole(buffer)),
            )],
        ))
        .unwrap()
}

fn create_render_attachment(
    device: &DeterministicRhiContractDevice,
    label: &str,
    format: TextureFormat,
) -> TextureHandle {
    device
        .create_texture(&TextureDesc::new(
            label,
            32,
            32,
            format,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap()
}

fn color_attachment(texture: TextureHandle) -> RenderPassColorAttachmentDesc {
    RenderPassColorAttachmentDesc::new(
        texture,
        RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
        RenderPassStoreOp::Store,
    )
}

fn depth_attachment(texture: TextureHandle) -> RenderPassDepthStencilAttachmentDesc {
    RenderPassDepthStencilAttachmentDesc::depth(
        texture,
        RenderPassDepthLoadOp::Clear(1.0),
        RenderPassStoreOp::Store,
    )
}

fn begin_default_render_pass(
    command_list: &mut dyn CommandList,
    color: TextureHandle,
    depth: TextureHandle,
) {
    command_list.begin_render_pass(
        "test-render-pass",
        vec![color_attachment(color)],
        Some(depth_attachment(depth)),
    );
}

fn create_raster_vertex_input_layout() -> VertexInputLayoutDesc {
    VertexInputLayoutDesc::new(vec![
        VertexBufferLayoutDesc::new(
            12,
            vec![VertexAttributeDesc::new(0, 0, VertexFormat::Float32x3)],
        ),
        VertexBufferLayoutDesc::new(
            16,
            vec![VertexAttributeDesc::new(1, 0, VertexFormat::Float32x4)],
        )
        .with_step_mode(VertexStepMode::Instance),
    ])
}

mod basic_commands;
mod bind_groups;
mod indirect_commands;
mod raster_draws;
mod vertex_index_state;
