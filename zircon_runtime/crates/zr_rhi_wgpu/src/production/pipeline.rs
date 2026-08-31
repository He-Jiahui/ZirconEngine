use zr_rhi::{PipelineDesc, RhiError};

use super::registry::WgpuResourceRegistry;
use super::translate::{
    wgpu_blend_state, wgpu_color_writes, wgpu_compare_function, wgpu_cull_mode, wgpu_front_face,
    wgpu_primitive_topology, wgpu_texture_format, wgpu_vertex_attribute, wgpu_vertex_step_mode,
};

/// Native pipeline construction is isolated from handle allocation and
/// descriptor lookup so the registry remains one generation's object table.
pub(super) fn create_compute_pipeline(
    device: &wgpu::Device,
    registry: &WgpuResourceRegistry,
    desc: &PipelineDesc,
) -> Result<wgpu::ComputePipeline, RhiError> {
    let layout = registry.pipeline_layout(desc.layout.ok_or_else(|| {
        invalid_pipeline_descriptor(desc, "compute pipeline requires a pipeline layout")
    })?)?;
    let shader_handle = desc.compute_shader.ok_or_else(|| {
        invalid_pipeline_descriptor(desc, "compute pipeline requires a compute shader")
    })?;
    let shader = registry.shader_module(shader_handle)?;
    let shader_desc = registry.shader_module_desc(shader_handle)?;
    Ok(
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: desc.label.as_deref(),
            layout: Some(layout),
            module: shader,
            entry_point: Some(&shader_desc.entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        }),
    )
}

pub(super) fn create_raster_pipeline(
    device: &wgpu::Device,
    registry: &WgpuResourceRegistry,
    desc: &PipelineDesc,
) -> Result<wgpu::RenderPipeline, RhiError> {
    let layout = registry.pipeline_layout(desc.layout.ok_or_else(|| {
        invalid_pipeline_descriptor(desc, "raster pipeline requires a pipeline layout")
    })?)?;
    let vertex_handle = desc.vertex_shader.ok_or_else(|| {
        invalid_pipeline_descriptor(desc, "raster pipeline requires a vertex shader")
    })?;
    let vertex_shader = registry.shader_module(vertex_handle)?;
    let vertex_desc = registry.shader_module_desc(vertex_handle)?;
    let raster_state = desc.raster_state.as_ref().ok_or_else(|| {
        invalid_pipeline_descriptor(desc, "raster pipeline requires raster state")
    })?;
    if matches!(
        raster_state.primitive.topology,
        zr_rhi::PrimitiveTopology::LineStrip | zr_rhi::PrimitiveTopology::TriangleStrip
    ) {
        return Err(invalid_pipeline_descriptor(
            desc,
            "strip topologies require the M7 index-format ABI",
        ));
    }
    if raster_state
        .depth_stencil
        .is_some_and(|depth_stencil| depth_stencil.stencil_enabled)
    {
        return Err(invalid_pipeline_descriptor(
            desc,
            "stencil state requires the M7 stencil ABI",
        ));
    }

    let vertex_attributes = raster_state
        .vertex_input
        .buffers
        .iter()
        .map(|buffer| {
            buffer
                .attributes
                .iter()
                .map(wgpu_vertex_attribute)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let vertex_buffers = raster_state
        .vertex_input
        .buffers
        .iter()
        .zip(&vertex_attributes)
        .map(|(buffer, attributes)| wgpu::VertexBufferLayout {
            array_stride: buffer.array_stride,
            step_mode: wgpu_vertex_step_mode(buffer.step_mode),
            attributes,
        })
        .collect::<Vec<_>>();
    let color_targets = raster_state
        .color_targets
        .iter()
        .map(|target| {
            Some(wgpu::ColorTargetState {
                format: wgpu_texture_format(target.format),
                blend: target.blend.map(wgpu_blend_state),
                write_mask: wgpu_color_writes(target.write_mask),
            })
        })
        .collect::<Vec<_>>();
    let fragment_shader = match desc.fragment_shader {
        Some(handle) => {
            let shader = registry.shader_module(handle)?;
            let shader_desc = registry.shader_module_desc(handle)?;
            Some((shader, shader_desc))
        }
        None => None,
    };
    let fragment = fragment_shader
        .as_ref()
        .map(|(shader, shader_desc)| wgpu::FragmentState {
            module: shader,
            entry_point: Some(&shader_desc.entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &color_targets,
        });
    let depth_stencil = raster_state
        .depth_stencil
        .map(|depth_stencil| wgpu::DepthStencilState {
            format: wgpu_texture_format(depth_stencil.format),
            depth_write_enabled: Some(depth_stencil.depth_write_enabled),
            depth_compare: Some(wgpu_compare_function(depth_stencil.depth_compare)),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

    Ok(
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: desc.label.as_deref(),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: vertex_shader,
                entry_point: Some(&vertex_desc.entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_buffers,
            },
            fragment,
            primitive: wgpu::PrimitiveState {
                topology: wgpu_primitive_topology(raster_state.primitive.topology),
                strip_index_format: None,
                front_face: wgpu_front_face(raster_state.primitive.front_face),
                cull_mode: wgpu_cull_mode(raster_state.primitive.cull_mode),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: raster_state.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        }),
    )
}

fn invalid_pipeline_descriptor(desc: &PipelineDesc, reason: &str) -> RhiError {
    RhiError::InvalidPipelineDescriptor {
        label: desc.label.clone(),
        reason: reason.to_string(),
    }
}
