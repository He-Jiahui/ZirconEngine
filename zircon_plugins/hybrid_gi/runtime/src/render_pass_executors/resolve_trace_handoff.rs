use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::RenderHybridGiDebugView;
use zircon_runtime::graphics::{RenderPassExecutionContext, RenderPassGpuExecutionContext};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;
use zircon_runtime::rhi::TextureFormat;

use super::{
    HYBRID_GI_HISTORY_RESOURCE, HYBRID_GI_LIGHTING_RESOURCE,
    HYBRID_GI_TEMPORAL_METADATA_HISTORY_RESOURCE, HYBRID_GI_TEMPORAL_METADATA_RESOURCE,
    HYBRID_GI_TRACE_RESOURCE, SCENE_VELOCITY_RESOURCE,
};

const HYBRID_GI_RESOLVE_TRACE_PIPELINE_LABEL: &str = "zircon-hybrid-gi-resolve-trace-depth-source";
const HYBRID_GI_TEMPORAL_HISTORY_WEIGHT: f32 = 0.9;
const HYBRID_GI_TEMPORAL_MOTION_REJECTION_SCALE: f32 = 32.0;
const HYBRID_GI_TEMPORAL_DEPTH_REJECTION_THRESHOLD: f32 = 0.02;
const HYBRID_GI_TEMPORAL_LUMA_REJECTION_THRESHOLD: f32 = 0.08;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct HybridGiTemporalResolveParams {
    viewport_and_flags: [u32; 4],
    blend_and_rejection: [f32; 4],
}

impl HybridGiTemporalResolveParams {
    fn new(
        viewport_size: [u32; 2],
        history_available: bool,
        debug_view: RenderHybridGiDebugView,
    ) -> Self {
        Self {
            viewport_and_flags: [
                viewport_size[0].max(1),
                viewport_size[1].max(1),
                u32::from(history_available && debug_view == RenderHybridGiDebugView::None),
                resolve_debug_view_code(debug_view),
            ],
            blend_and_rejection: [
                HYBRID_GI_TEMPORAL_HISTORY_WEIGHT,
                HYBRID_GI_TEMPORAL_MOTION_REJECTION_SCALE,
                HYBRID_GI_TEMPORAL_DEPTH_REJECTION_THRESHOLD,
                HYBRID_GI_TEMPORAL_LUMA_REJECTION_THRESHOLD,
            ],
        }
    }
}

const fn resolve_debug_view_code(debug_view: RenderHybridGiDebugView) -> u32 {
    match debug_view {
        RenderHybridGiDebugView::None => 0,
        RenderHybridGiDebugView::Cards => 1,
        RenderHybridGiDebugView::SurfaceCache => 2,
        RenderHybridGiDebugView::VoxelClipmap => 3,
        RenderHybridGiDebugView::InputSet => 4,
    }
}

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
    let scene_velocity_view = gpu
        .require_texture_view(SCENE_VELOCITY_RESOURCE, RenderGraphResourceAccessKind::Read)?
        .clone();
    let history_view = gpu
        .require_texture_view(
            HYBRID_GI_HISTORY_RESOURCE,
            RenderGraphResourceAccessKind::Read,
        )?
        .clone();
    let temporal_metadata_history_view = gpu
        .require_texture_view(
            HYBRID_GI_TEMPORAL_METADATA_HISTORY_RESOURCE,
            RenderGraphResourceAccessKind::Read,
        )?
        .clone();
    let lighting_view = gpu
        .require_texture_view(
            HYBRID_GI_LIGHTING_RESOURCE,
            RenderGraphResourceAccessKind::Write,
        )?
        .clone();
    let temporal_metadata_view = gpu
        .require_texture_view(
            HYBRID_GI_TEMPORAL_METADATA_RESOURCE,
            RenderGraphResourceAccessKind::Write,
        )?
        .clone();
    let lighting_desc = gpu.require_texture_desc(
        HYBRID_GI_LIGHTING_RESOURCE,
        RenderGraphResourceAccessKind::Write,
    )?;
    let lighting_format = wgpu_texture_format(lighting_desc.format)?;
    let lighting_sample_count = lighting_desc.sample_count.max(1);
    let temporal_metadata_desc = gpu.require_texture_desc(
        HYBRID_GI_TEMPORAL_METADATA_RESOURCE,
        RenderGraphResourceAccessKind::Write,
    )?;
    let temporal_metadata_format = wgpu_texture_format(temporal_metadata_desc.format)?;
    let temporal_metadata_sample_count = temporal_metadata_desc.sample_count.max(1);
    if lighting_sample_count != temporal_metadata_sample_count {
        return Err(format!(
            "hybrid GI temporal resolve attachment sample mismatch: lighting={}, metadata={}",
            lighting_sample_count, temporal_metadata_sample_count
        ));
    }
    let viewport_size = gpu.viewport_size();
    let debug_view = gpu
        .frame_extract()
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
        .map_or(RenderHybridGiDebugView::None, |extract| extract.debug_view);
    let params = HybridGiTemporalResolveParams::new(
        [viewport_size.x, viewport_size.y],
        gpu.hybrid_gi_history_available(),
        debug_view,
    );
    let params_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-hybrid-gi-temporal-resolve-params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
    encode_resolve_trace_handoff(
        gpu,
        &hybrid_gi_trace_buffer,
        &scene_velocity_view,
        &history_view,
        &temporal_metadata_history_view,
        &params_buffer,
        &lighting_view,
        &temporal_metadata_view,
        lighting_format,
        temporal_metadata_format,
        lighting_sample_count,
    )
}

fn encode_resolve_trace_handoff(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
    scene_velocity_view: &wgpu::TextureView,
    history_view: &wgpu::TextureView,
    temporal_metadata_history_view: &wgpu::TextureView,
    params_buffer: &wgpu::Buffer,
    lighting_view: &wgpu::TextureView,
    temporal_metadata_view: &wgpu::TextureView,
    lighting_format: wgpu::TextureFormat,
    temporal_metadata_format: wgpu::TextureFormat,
    lighting_sample_count: u32,
) -> Result<(), String> {
    let bind_group_layout = create_resolve_trace_bind_group_layout(gpu.device);
    let pipeline = create_resolve_trace_pipeline(
        gpu.device,
        &bind_group_layout,
        lighting_format,
        temporal_metadata_format,
        lighting_sample_count,
    );
    let bind_group = create_resolve_trace_bind_group(
        gpu.device,
        &bind_group_layout,
        hybrid_gi_trace_buffer,
        scene_velocity_view,
        history_view,
        temporal_metadata_history_view,
        params_buffer,
    );
    let mut pass = gpu.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("HybridGiResolveTraceDepthSourcePass"),
        color_attachments: &[
            Some(resolve_color_attachment(lighting_view)),
            Some(resolve_color_attachment(temporal_metadata_view)),
        ],
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

fn resolve_color_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    }
}

fn create_resolve_trace_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-resolve-trace-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            texture_layout_entry(1),
            texture_layout_entry(2),
            texture_layout_entry(3),
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn create_resolve_trace_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    lighting_format: wgpu::TextureFormat,
    temporal_metadata_format: wgpu::TextureFormat,
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
            targets: &[
                Some(resolve_color_target(lighting_format)),
                Some(resolve_color_target(temporal_metadata_format)),
            ],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn resolve_color_target(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

fn create_resolve_trace_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
    scene_velocity_view: &wgpu::TextureView,
    history_view: &wgpu::TextureView,
    temporal_metadata_history_view: &wgpu::TextureView,
    params_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-resolve-trace-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: hybrid_gi_trace_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(scene_velocity_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(history_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(temporal_metadata_history_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: params_buffer.as_entire_binding(),
            },
        ],
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
mod tests;
