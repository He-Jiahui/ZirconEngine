use std::sync::Arc;

use crate::graphics::scene::scene_renderer::core::DEPTH_FORMAT;
use crate::graphics::scene::scene_renderer::overlay::ViewportIconSource;
use crate::graphics::scene::scene_renderer::primitives::IconVertex;

use super::scene_gizmo_pass::SceneGizmoPass;

const ICON_SHADER: &str = include_str!("../../shaders/icon.wgsl");

impl SceneGizmoPass {
    pub(crate) fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Self {
        let icon_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-icon-layout"),
            bind_group_layouts: &[Some(scene_layout), Some(texture_layout)],
            immediate_size: 0,
        });
        let icon_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-icon-shader"),
            source: wgpu::ShaderSource::Wgsl(ICON_SHADER.into()),
        });
        let icon_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-icon-pipeline"),
            layout: Some(&icon_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &icon_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[IconVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &icon_shader,
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
        });
        let icon_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-icon-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Self {
            icon_pipeline,
            icon_sampler,
            icon_atlas: crate::graphics::scene::scene_renderer::overlay::ViewportIconAtlas::new(
                icon_source,
            ),
        }
    }
}
