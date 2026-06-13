use wgpu::util::DeviceExt;

use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
};

use super::build_sprite_vertices::build_sprite_vertices;
use super::prepared_batches::prepare_sprite_draw_batches;
use super::sprite_vertex::SpriteVertex;

const SPRITE_SHADER: &str = r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = scene.view_proj * vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, input.uv) * input.color;
}
"#;

pub(crate) struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl SpriteRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-sprite-shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-sprite-layout"),
            bind_group_layouts: &[Some(scene_layout), Some(texture_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-sprite-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[SpriteVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::super::core::DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self { pipeline }
    }

    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        stage: RenderPassStage,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) {
        let sprite_batches =
            prepare_sprite_draw_batches(frame, build_sprite_vertices(frame, stage));
        if sprite_batches.is_empty() {
            return;
        }

        let sprite_draw_count = sprite_batches.len();
        for (draw_index, batch) in sprite_batches.into_iter().enumerate() {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-sprite-vertices"),
                contents: bytemuck::cast_slice(batch.vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let texture = streamer.texture(Some(batch.texture_id()));
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SpritePass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(
                        sprite_subpass_attachment_ops(
                            attachment_ops,
                            draw_index,
                            sprite_draw_count,
                        ),
                        wgpu::Color::TRANSPARENT,
                    ),
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(depth_attachment_operations(
                        sprite_subpass_attachment_ops(
                            depth_attachment_ops,
                            draw_index,
                            sprite_draw_count,
                        ),
                        1.0,
                    )),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, scene_bind_group, &[]);
            pass.set_bind_group(1, &texture.bind_group, &[]);
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..batch.vertices().len() as u32, 0..1);
        }
    }
}

fn sprite_subpass_attachment_ops(
    graph_pass_ops: RenderGraphAttachmentOps,
    draw_index: usize,
    draw_count: usize,
) -> RenderGraphAttachmentOps {
    RenderGraphAttachmentOps {
        load: if draw_index == 0 {
            graph_pass_ops.load
        } else {
            RenderGraphAttachmentLoadOp::Load
        },
        store: if draw_index + 1 == draw_count {
            graph_pass_ops.store
        } else {
            RenderGraphAttachmentStoreOp::Store
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::render_graph::{
        RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    };

    use super::sprite_subpass_attachment_ops;

    #[test]
    fn sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws() {
        let graph_ops = RenderGraphAttachmentOps::clear_discard();

        assert_eq!(
            sprite_subpass_attachment_ops(graph_ops, 0, 3),
            RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Clear,
                store: RenderGraphAttachmentStoreOp::Store,
            }
        );
        assert_eq!(
            sprite_subpass_attachment_ops(graph_ops, 1, 3),
            RenderGraphAttachmentOps::load_store()
        );
        assert_eq!(
            sprite_subpass_attachment_ops(graph_ops, 2, 3),
            RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Load,
                store: RenderGraphAttachmentStoreOp::Discard,
            }
        );
    }
}
