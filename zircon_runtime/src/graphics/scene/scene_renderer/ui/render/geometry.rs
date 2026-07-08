use bytemuck::{Pod, Zeroable};
use zircon_runtime_interface::ui::layout::UiFrame;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
}

impl ScreenSpaceUiVertex {
    pub(in crate::graphics::scene::scene_renderer::ui) fn layout(
    ) -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ScreenSpaceUiScissor {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) fn push_rect(
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

pub(super) fn frame_to_scissor(frame: UiFrame) -> Option<ScreenSpaceUiScissor> {
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
