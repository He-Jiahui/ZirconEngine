use std::sync::Arc;

use crate::scene::scene_renderer::core::DEPTH_FORMAT;
use crate::scene::scene_renderer::overlay::{
    begin_line_pass, PreparedIconDraw, PreparedSceneGizmoPass, ViewportIconAtlas,
    ViewportIconSource,
};
use crate::scene::scene_renderer::primitives::{
    build_icon_buffer, build_icon_quad_vertices, build_line_buffer,
    build_scene_gizmo_line_vertices, IconVertex,
};
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

const ICON_SHADER: &str = include_str!("../shaders/icon.wgsl");

pub(crate) struct SceneGizmoPass {
    icon_pipeline: wgpu::RenderPipeline,
    icon_sampler: wgpu::Sampler,
    icon_atlas: ViewportIconAtlas,
}

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
            bind_group_layouts: &[scene_layout, texture_layout],
            push_constant_ranges: &[],
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
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
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
            multiview: None,
            cache: None,
        });
        let icon_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-icon-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            icon_pipeline,
            icon_sampler,
            icon_atlas: ViewportIconAtlas::new(icon_source),
        }
    }

    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<PreparedSceneGizmoPass, GraphicsError> {
        let camera = &frame.scene.scene.camera;
        let camera_right = camera.transform.right();
        let camera_up = camera.transform.up();
        let mut icon_draws = Vec::new();

        for gizmo in &frame.scene.overlays.scene_gizmos {
            for icon in &gizmo.icons {
                let Some(bind_group) = self.icon_atlas.ensure(
                    icon.id,
                    device,
                    queue,
                    texture_layout,
                    &self.icon_sampler,
                )?
                else {
                    continue;
                };
                let vertices = build_icon_quad_vertices(icon, camera_right, camera_up);
                if let Some((vertex_buffer, vertex_count)) =
                    build_icon_buffer(device, "zircon-scene-gizmo-icon-buffer", &vertices)
                {
                    icon_draws.push(PreparedIconDraw {
                        bind_group,
                        vertex_buffer,
                        vertex_count,
                    });
                }
            }
        }

        let line_vertices = build_scene_gizmo_line_vertices(frame, |id| self.icon_atlas.has(id));
        Ok(PreparedSceneGizmoPass {
            line_buffer: build_line_buffer(device, "zircon-scene-gizmo-buffer", &line_vertices),
            icon_draws,
        })
    }

    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        line_pipeline: &wgpu::RenderPipeline,
        line_buffer: Option<&(wgpu::Buffer, u32)>,
        icon_draws: &[PreparedIconDraw],
    ) {
        if line_buffer.is_none() && icon_draws.is_empty() {
            return;
        }
        let mut pass = begin_line_pass(encoder, "SceneGizmoPass", color_view, depth_view);
        pass.set_bind_group(0, scene_bind_group, &[]);
        if let Some((buffer, count)) = line_buffer {
            pass.set_pipeline(line_pipeline);
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..*count, 0..1);
        }
        if !icon_draws.is_empty() {
            pass.set_pipeline(&self.icon_pipeline);
            for draw in icon_draws {
                pass.set_bind_group(1, draw.bind_group.as_ref(), &[]);
                pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
                pass.draw(0..draw.vertex_count, 0..1);
            }
        }
    }
}
