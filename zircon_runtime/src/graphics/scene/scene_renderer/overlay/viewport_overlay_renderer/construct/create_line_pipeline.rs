use super::super::super::super::core::DEPTH_FORMAT;
use super::super::super::super::primitives::LineVertex;

const LINE_SHADER: &str = include_str!("../../shaders/line.wgsl");

pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) fn create_line_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    scene_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let line_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-line-layout"),
        bind_group_layouts: &[Some(scene_layout)],
        immediate_size: 0,
    });
    let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-line-shader"),
        source: wgpu::ShaderSource::Wgsl(LINE_SHADER.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-line-pipeline"),
        layout: Some(&line_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &line_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[LineVertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &line_shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
