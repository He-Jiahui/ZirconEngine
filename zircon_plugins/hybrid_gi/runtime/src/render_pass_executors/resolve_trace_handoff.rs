use zircon_runtime::graphics::{RenderPassExecutionContext, RenderPassGpuExecutionContext};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;
use zircon_runtime::rhi::TextureFormat;

use super::{HYBRID_GI_LIGHTING_RESOURCE, HYBRID_GI_TRACE_RESOURCE};

const HYBRID_GI_RESOLVE_TRACE_PIPELINE_LABEL: &str = "zircon-hybrid-gi-resolve-trace-depth-source";

pub(super) fn record_resolve_trace_handoff(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let gpu = context.require_gpu()?;
    let hybrid_gi_trace_buffer = gpu
        .require_buffer(
            HYBRID_GI_TRACE_RESOURCE,
            RenderGraphResourceAccessKind::Read,
        )?
        .clone();
    let lighting_desc = gpu.require_texture_desc(
        HYBRID_GI_LIGHTING_RESOURCE,
        RenderGraphResourceAccessKind::Write,
    )?;
    let lighting_format = wgpu_texture_format(lighting_desc.format)?;
    let lighting_sample_count = lighting_desc.sample_count.max(1);
    encode_resolve_trace_handoff(
        gpu,
        &hybrid_gi_trace_buffer,
        lighting_format,
        lighting_sample_count,
    )
}

fn encode_resolve_trace_handoff(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
    lighting_format: wgpu::TextureFormat,
    lighting_sample_count: u32,
) -> Result<(), String> {
    let lighting_view = gpu
        .require_texture_view(
            HYBRID_GI_LIGHTING_RESOURCE,
            RenderGraphResourceAccessKind::Write,
        )?
        .clone();
    let bind_group_layout = create_resolve_trace_bind_group_layout(gpu.device);
    let pipeline = create_resolve_trace_pipeline(
        gpu.device,
        &bind_group_layout,
        lighting_format,
        lighting_sample_count,
    );
    let bind_group =
        create_resolve_trace_bind_group(gpu.device, &bind_group_layout, hybrid_gi_trace_buffer);
    let mut pass = gpu.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("HybridGiResolveTraceDepthSourcePass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &lighting_view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..3, 0..1);
    Ok(())
}

fn create_resolve_trace_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-resolve-trace-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_resolve_trace_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    lighting_format: wgpu::TextureFormat,
    lighting_sample_count: u32,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-resolve-trace-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../hybrid_gi/renderer/shaders/resolve_trace_depth_source.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-resolve-trace-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(HYBRID_GI_RESOLVE_TRACE_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: lighting_sample_count,
            ..wgpu::MultisampleState::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: lighting_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn create_resolve_trace_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-resolve-trace-bind-group"),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: hybrid_gi_trace_buffer.as_entire_binding(),
        }],
    })
}

fn wgpu_texture_format(format: TextureFormat) -> Result<wgpu::TextureFormat, String> {
    let format = match format {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
        TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
        TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        TextureFormat::Rg11b10Ufloat => wgpu::TextureFormat::Rg11b10Ufloat,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::Depth24Plus
        | TextureFormat::Depth24PlusStencil8
        | TextureFormat::Depth32Float => {
            return Err(format!(
                "hybrid GI resolve target `{HYBRID_GI_LIGHTING_RESOURCE}` must be a color texture, got `{format:?}`"
            ));
        }
    };
    Ok(format)
}

#[cfg(test)]
mod tests {
    #[test]
    fn resolve_shader_consumes_trace_depth_source_packet() {
        let source = include_str!("../hybrid_gi/renderer/shaders/resolve_trace_depth_source.wgsl");

        assert!(source.contains("HYBRID_GI_TRACE_SCHEDULE_MAGIC"));
        assert!(source.contains("hybrid_gi_trace_words[6]"));
        assert!(source.contains("@fragment"));
        assert!(source.contains("unpack_rgba8"));
    }
}
