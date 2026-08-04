use crate::text::atlas::render_contract::{GLYPH_ATLAS_TEXT_SHADER, GlyphAtlasBlendMode};
use crate::text::atlas::render_gpu_plan::{
    GlyphAtlasGpuPipelineContract, GlyphAtlasGpuPrimitiveTopology,
};

use super::instance::glyph_atlas_wgpu_instance_buffer_layout;

pub(super) fn create_glyph_atlas_bitmap_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-screen-space-ui-glyph-atlas-shader"),
        source: wgpu::ShaderSource::Wgsl(GLYPH_ATLAS_TEXT_SHADER.into()),
    })
}

pub(super) fn create_glyph_atlas_bitmap_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-screen-space-ui-glyph-atlas-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    })
}

pub(super) fn create_glyph_atlas_bitmap_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    target_format: wgpu::TextureFormat,
    contract: GlyphAtlasGpuPipelineContract,
) -> wgpu::RenderPipeline {
    let label = glyph_atlas_bitmap_pipeline_label(contract);
    let instance_layout = glyph_atlas_wgpu_instance_buffer_layout(contract.instance_layout);
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label.as_str()),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(contract.shader_entry_points.vertex),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[instance_layout],
        },
        primitive: glyph_atlas_wgpu_primitive_state(contract.key.primitive_topology),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(contract.shader_entry_points.fragment),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(glyph_atlas_wgpu_blend_state(
                    contract.key.render_contract.blend_mode,
                )),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn glyph_atlas_wgpu_blend_state(blend_mode: GlyphAtlasBlendMode) -> wgpu::BlendState {
    match blend_mode {
        GlyphAtlasBlendMode::StandardAlpha | GlyphAtlasBlendMode::SourceRgba => {
            wgpu::BlendState::ALPHA_BLENDING
        }
        GlyphAtlasBlendMode::SubpixelBackgroundComposite => wgpu::BlendState {
            color: wgpu::BlendComponent::REPLACE,
            alpha: wgpu::BlendComponent::REPLACE,
        },
    }
}

pub(super) fn glyph_atlas_wgpu_primitive_state(
    topology: GlyphAtlasGpuPrimitiveTopology,
) -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: glyph_atlas_wgpu_primitive_topology(topology),
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
    }
}

fn glyph_atlas_wgpu_primitive_topology(
    topology: GlyphAtlasGpuPrimitiveTopology,
) -> wgpu::PrimitiveTopology {
    match topology {
        GlyphAtlasGpuPrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
    }
}

fn glyph_atlas_bitmap_pipeline_label(contract: GlyphAtlasGpuPipelineContract) -> String {
    format!(
        "zircon-screen-space-ui-glyph-atlas-{}-pipeline",
        contract.shader_entry_points.fragment
    )
}
