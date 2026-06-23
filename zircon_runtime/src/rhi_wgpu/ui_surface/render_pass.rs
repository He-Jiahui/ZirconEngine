use std::collections::HashMap;

use wgpu::util::DeviceExt;

use super::batching::DrawOp;
use super::geometry::{ImageVertex, SolidVertex};
use super::text::WgpuUiTextRenderer;
use super::WgpuUiImageResource;

pub(super) struct WgpuUiDrawBuffers {
    solid: Option<wgpu::Buffer>,
    image: Option<wgpu::Buffer>,
}

impl WgpuUiDrawBuffers {
    pub(super) fn new(device: &wgpu::Device, draw_ops: &[DrawOp]) -> Self {
        let solid_vertices = draw_ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Solid(draw) => Some(draw.vertices.as_slice()),
                DrawOp::Image(_) | DrawOp::Text(_) => None,
            })
            .flatten()
            .copied()
            .collect::<Vec<SolidVertex>>();
        let image_vertices = draw_ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Image(draw) => Some(draw.vertices.as_slice()),
                DrawOp::Solid(_) | DrawOp::Text(_) => None,
            })
            .flatten()
            .copied()
            .collect::<Vec<ImageVertex>>();
        let solid = (!solid_vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-ui-solid-vertices"),
                contents: bytemuck::cast_slice(&solid_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        let image = (!image_vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-ui-image-vertices"),
                contents: bytemuck::cast_slice(&image_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        Self { solid, image }
    }
}

#[derive(Clone, Copy)]
pub(super) enum TargetLoad {
    ClearBlack,
    Load,
}

impl TargetLoad {
    fn load_op(self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            Self::ClearBlack => wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            Self::Load => wgpu::LoadOp::Load,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_draw_ops_to_view(
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    initial_load: TargetLoad,
    surface_size: (u32, u32),
    draw_ops: &[DrawOp],
    buffers: &WgpuUiDrawBuffers,
    solid_pipeline: &wgpu::RenderPipeline,
    image_pipeline: &wgpu::RenderPipeline,
    image_cache: &HashMap<String, WgpuUiImageResource>,
    text: &mut WgpuUiTextRenderer,
) {
    if draw_ops.is_empty() {
        let mut pass = begin_ui_surface_pass(encoder, target_view, initial_load);
        set_surface_viewport(&mut pass, surface_size);
        return;
    }

    let mut first_pass = true;
    let mut op_index = 0;
    while op_index < draw_ops.len() {
        let load = if first_pass {
            initial_load
        } else {
            TargetLoad::Load
        };
        let mut pass = begin_ui_surface_pass(encoder, target_view, load);
        set_surface_viewport(&mut pass, surface_size);
        first_pass = false;
        match &draw_ops[op_index] {
            DrawOp::Solid(_) => {
                let Some(buffer) = buffers.solid.as_ref() else {
                    op_index += 1;
                    continue;
                };
                pass.set_pipeline(solid_pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                let DrawOp::Solid(draw) = &draw_ops[op_index] else {
                    unreachable!("draw op kind checked above");
                };
                pass.draw(draw.vertex_start..draw.vertex_end, 0..1);
                op_index += 1;
            }
            DrawOp::Image(_) => {
                let Some(buffer) = buffers.image.as_ref() else {
                    op_index += 1;
                    continue;
                };
                pass.set_pipeline(image_pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                let DrawOp::Image(draw) = &draw_ops[op_index] else {
                    unreachable!("draw op kind checked above");
                };
                if let Some(resource) = image_cache.get(&draw.resource_key) {
                    pass.set_bind_group(0, &resource.bind_group, &[]);
                    pass.draw(draw.vertex_start..draw.vertex_end, 0..1);
                }
                op_index += 1;
            }
            DrawOp::Text(draw) => {
                text.render_batch(draw.batch_index, &mut pass);
                op_index += 1;
            }
        }
    }
}

fn begin_ui_surface_pass<'encoder>(
    encoder: &'encoder mut wgpu::CommandEncoder,
    target_view: &'encoder wgpu::TextureView,
    load: TargetLoad,
) -> wgpu::RenderPass<'encoder> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zircon-ui-surface-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load.load_op(),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

fn set_surface_viewport(pass: &mut wgpu::RenderPass<'_>, surface_size: (u32, u32)) {
    pass.set_viewport(
        0.0,
        0.0,
        surface_size.0 as f32,
        surface_size.1 as f32,
        0.0,
        1.0,
    );
}
