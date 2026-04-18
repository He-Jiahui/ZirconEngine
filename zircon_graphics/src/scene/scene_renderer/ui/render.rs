use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use zircon_ui::{UiFrame, UiRenderCommand, UiRenderCommandKind};

use crate::types::EditorOrRuntimeFrame;

use super::screen_space_ui_renderer::ScreenSpaceUiRenderer;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ScreenSpaceUiVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl ScreenSpaceUiVertex {
    pub(super) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

struct PreparedScreenSpaceUi {
    vertex_buffer: wgpu::Buffer,
    draws: Vec<ScreenSpaceUiDraw>,
}

struct ScreenSpaceUiDraw {
    vertices: Range<u32>,
    scissor: ScreenSpaceUiScissor,
}

#[derive(Clone, Copy)]
struct ScreenSpaceUiScissor {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ScreenSpaceUiRenderer {
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        frame: &EditorOrRuntimeFrame,
    ) {
        let Some(prepared) = prepare_screen_space_ui(device, frame) else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-screen-space-ui-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, prepared.vertex_buffer.slice(..));

        for draw in &prepared.draws {
            pass.set_scissor_rect(
                draw.scissor.x,
                draw.scissor.y,
                draw.scissor.width,
                draw.scissor.height,
            );
            pass.draw(draw.vertices.clone(), 0..1);
        }
    }
}

fn prepare_screen_space_ui(
    device: &wgpu::Device,
    frame: &EditorOrRuntimeFrame,
) -> Option<PreparedScreenSpaceUi> {
    let extract = frame.ui.as_ref()?;
    let viewport = UiFrame::new(
        0.0,
        0.0,
        frame.viewport_size.x.max(1) as f32,
        frame.viewport_size.y.max(1) as f32,
    );
    let full_scissor = ScreenSpaceUiScissor {
        x: 0,
        y: 0,
        width: frame.viewport_size.x.max(1),
        height: frame.viewport_size.y.max(1),
    };

    let mut vertices = Vec::new();
    let mut draws = Vec::new();
    for command in &extract.list.commands {
        let start = vertices.len() as u32;
        push_command_geometry(command, viewport, &mut vertices);
        let end = vertices.len() as u32;
        if end > start {
            let scissor = command
                .clip_frame
                .and_then(|clip| viewport.intersection(clip))
                .and_then(frame_to_scissor)
                .unwrap_or(full_scissor);
            draws.push(ScreenSpaceUiDraw {
                vertices: start..end,
                scissor,
            });
        }
    }

    (!vertices.is_empty()).then(|| PreparedScreenSpaceUi {
        vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-screen-space-ui-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        draws,
    })
}

fn push_command_geometry(
    command: &UiRenderCommand,
    viewport: UiFrame,
    vertices: &mut Vec<ScreenSpaceUiVertex>,
) {
    if command.opacity <= 0.0 {
        return;
    }

    let frame = match viewport.intersection(command.frame) {
        Some(frame) => frame,
        None => return,
    };

    match command.kind {
        UiRenderCommandKind::Group => {}
        UiRenderCommandKind::Quad => {
            if let Some(color) = parse_color(
                command.style.background_color.as_deref(),
                [0.16, 0.19, 0.24, 1.0],
                command.opacity,
            ) {
                push_rect(vertices, frame, color, viewport);
            }
            let border_width = command.style.border_width.max(0.0);
            if border_width > 0.0 {
                let color = parse_color(
                    command.style.border_color.as_deref(),
                    [0.85, 0.88, 0.92, 1.0],
                    command.opacity,
                )
                .unwrap_or([0.85, 0.88, 0.92, command.opacity]);
                push_border(vertices, frame, border_width, color, viewport);
            }
        }
        UiRenderCommandKind::Text => {
            let band_height = frame.height.clamp(4.0, 12.0);
            let inset = (frame.height * 0.2).min(10.0);
            let band = UiFrame::new(
                frame.x + inset,
                frame.y + (frame.height - band_height) * 0.5,
                (frame.width - inset * 2.0).max(4.0),
                band_height,
            );
            let color = parse_color(
                command.style.foreground_color.as_deref(),
                [0.96, 0.96, 0.96, 1.0],
                command.opacity,
            )
            .unwrap_or([0.96, 0.96, 0.96, command.opacity]);
            push_rect(vertices, band, color, viewport);
        }
        UiRenderCommandKind::Image => {
            let extent = (frame.width.min(frame.height) * 0.68).max(8.0);
            let icon = UiFrame::new(
                frame.x + (frame.width - extent) * 0.5,
                frame.y + (frame.height - extent) * 0.5,
                extent,
                extent,
            );
            let color = parse_color(
                command.style.foreground_color.as_deref(),
                [0.76, 0.88, 0.98, 1.0],
                command.opacity,
            )
            .unwrap_or([0.76, 0.88, 0.98, command.opacity]);
            push_rect(vertices, icon, color, viewport);
        }
    }
}

fn push_border(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    border_width: f32,
    color: [f32; 4],
    viewport: UiFrame,
) {
    let width = border_width
        .min(frame.width * 0.5)
        .min(frame.height * 0.5)
        .max(1.0);

    push_rect(
        vertices,
        UiFrame::new(frame.x, frame.y, frame.width, width),
        color,
        viewport,
    );
    push_rect(
        vertices,
        UiFrame::new(frame.x, frame.bottom() - width, frame.width, width),
        color,
        viewport,
    );
    if frame.height > width * 2.0 {
        push_rect(
            vertices,
            UiFrame::new(frame.x, frame.y + width, width, frame.height - width * 2.0),
            color,
            viewport,
        );
        push_rect(
            vertices,
            UiFrame::new(
                frame.right() - width,
                frame.y + width,
                width,
                frame.height - width * 2.0,
            ),
            color,
            viewport,
        );
    }
}

fn push_rect(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    color: [f32; 4],
    viewport: UiFrame,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    let x0 = pixel_to_ndc_x(frame.x, viewport.width);
    let x1 = pixel_to_ndc_x(frame.right(), viewport.width);
    let y0 = pixel_to_ndc_y(frame.y, viewport.height);
    let y1 = pixel_to_ndc_y(frame.bottom(), viewport.height);

    vertices.extend_from_slice(&[
        ScreenSpaceUiVertex {
            position: [x0, y0],
            color,
        },
        ScreenSpaceUiVertex {
            position: [x1, y0],
            color,
        },
        ScreenSpaceUiVertex {
            position: [x1, y1],
            color,
        },
        ScreenSpaceUiVertex {
            position: [x0, y0],
            color,
        },
        ScreenSpaceUiVertex {
            position: [x1, y1],
            color,
        },
        ScreenSpaceUiVertex {
            position: [x0, y1],
            color,
        },
    ]);
}

fn frame_to_scissor(frame: UiFrame) -> Option<ScreenSpaceUiScissor> {
    let x = frame.x.max(0.0).floor() as u32;
    let y = frame.y.max(0.0).floor() as u32;
    let width = frame.width.max(0.0).ceil() as u32;
    let height = frame.height.max(0.0).ceil() as u32;
    (width > 0 && height > 0).then_some(ScreenSpaceUiScissor {
        x,
        y,
        width,
        height,
    })
}

fn pixel_to_ndc_x(x: f32, width: f32) -> f32 {
    (x / width.max(1.0)) * 2.0 - 1.0
}

fn pixel_to_ndc_y(y: f32, height: f32) -> f32 {
    1.0 - (y / height.max(1.0)) * 2.0
}

fn parse_color(value: Option<&str>, fallback: [f32; 4], opacity: f32) -> Option<[f32; 4]> {
    parse_hex_color(value.unwrap_or(""), opacity).or_else(|| {
        (opacity > 0.0).then_some([fallback[0], fallback[1], fallback[2], fallback[3] * opacity])
    })
}

fn parse_hex_color(value: &str, opacity: f32) -> Option<[f32; 4]> {
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = parse_hex_byte(&hex[0..2])?;
            let g = parse_hex_byte(&hex[2..4])?;
            let b = parse_hex_byte(&hex[4..6])?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                opacity,
            ])
        }
        8 => {
            let r = parse_hex_byte(&hex[0..2])?;
            let g = parse_hex_byte(&hex[2..4])?;
            let b = parse_hex_byte(&hex[4..6])?;
            let a = parse_hex_byte(&hex[6..8])?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                (a as f32 / 255.0) * opacity,
            ])
        }
        _ => None,
    }
}

fn parse_hex_byte(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
