use zircon_runtime::graphics::{
    RenderPassExecutionContext, RenderPassGpuNativeContext, RenderPassGpuResourceFactory,
};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use crate::{
    HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS, HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
    HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
};

use super::{HYBRID_GI_SCENE_RESOURCE, HYBRID_GI_TRACE_RESOURCE, SCENE_HZB_RESOURCE};

pub(super) fn record_trace_schedule_handoff(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    let hybrid_gi_scene_buffer = gpu.require_buffer_binding(
        HYBRID_GI_SCENE_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    let hybrid_gi_trace_buffer = gpu.require_buffer_binding(
        HYBRID_GI_TRACE_RESOURCE,
        RenderGraphResourceAccessKind::Write,
    )?;
    // HZB 光线步进会读取多个 mip 级别，此处必须绑定覆盖完整 mip 链的视图。
    let scene_hzb_view = gpu.require_owned_texture_full_mip_view(
        SCENE_HZB_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    let mut native = gpu.native_context();
    encode_trace_schedule_handoff(
        &mut native,
        hybrid_gi_scene_buffer,
        hybrid_gi_trace_buffer,
        &scene_hzb_view,
    );
    drop(native);
    gpu.record_compute_dispatch(
        pass_name,
        executor_id,
        HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
        HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS,
        vec![HYBRID_GI_TRACE_RESOURCE.to_string()],
    );
    Ok(())
}

fn encode_trace_schedule_handoff(
    native: &mut RenderPassGpuNativeContext<'_, '_>,
    hybrid_gi_scene_buffer: wgpu::BufferBinding<'_>,
    hybrid_gi_trace_buffer: wgpu::BufferBinding<'_>,
    scene_hzb_view: &wgpu::TextureView,
) {
    let bind_group_layout = create_trace_schedule_handoff_bind_group_layout(native);
    let pipeline = create_trace_schedule_handoff_pipeline(native, &bind_group_layout);
    let bind_group = create_trace_schedule_handoff_bind_group(
        native,
        &bind_group_layout,
        hybrid_gi_scene_buffer,
        hybrid_gi_trace_buffer,
        scene_hzb_view,
    );
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiTraceScheduleHandoffPass"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[0],
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[1],
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[2],
    );
}

fn create_trace_schedule_handoff_bind_group_layout(
    factory: &impl RenderPassGpuResourceFactory,
) -> wgpu::BindGroupLayout {
    factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn create_trace_schedule_handoff_pipeline(
    factory: &impl RenderPassGpuResourceFactory,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_module = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../hybrid_gi/renderer/shaders/trace_schedule_handoff.wgsl").into(),
        ),
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_trace_schedule_handoff_bind_group(
    factory: &impl RenderPassGpuResourceFactory,
    bind_group_layout: &wgpu::BindGroupLayout,
    hybrid_gi_scene_buffer: wgpu::BufferBinding<'_>,
    hybrid_gi_trace_buffer: wgpu::BufferBinding<'_>,
    scene_hzb_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(hybrid_gi_scene_buffer),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(hybrid_gi_trace_buffer),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(scene_hzb_view),
            },
        ],
    })
}

#[cfg(test)]
#[path = "tests/trace_schedule_handoff.rs"]
mod tests;
