use bytemuck::{Pod, Zeroable};
use zircon_runtime_interface::ui::layout::UiFrame;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
    pub(super) local_position: [f32; 2],
    pub(super) half_extent: [f32; 2],
    pub(super) corner_radius: f32,
    pub(super) border_width: f32,
    pub(super) fill_color: [f32; 4],
}

impl ScreenSpaceUiVertex {
    pub(in crate::graphics::scene::scene_renderer::ui) fn layout(
    ) -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x4,
            2 => Float32x2,
            3 => Float32x2,
            4 => Float32,
            5 => Float32,
            6 => Float32x4,
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiScissor {
    pub(in crate::graphics::scene::scene_renderer::ui) x: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) y: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) width: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) height: u32,
}

pub(super) fn push_rect(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    color: [f32; 4],
    viewport: UiFrame,
) {
    push_rect_with_radius(vertices, frame, color, 0.0, viewport);
}

pub(super) fn push_rect_with_radius(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    color: [f32; 4],
    corner_radius: f32,
    viewport: UiFrame,
) {
    push_rect_geometry(vertices, frame, color, corner_radius, viewport, false);
}

fn push_rect_geometry(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    color: [f32; 4],
    corner_radius: f32,
    viewport: UiFrame,
    force_analytic: bool,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    let analytic = force_analytic
        || corner_radius.is_finite() && corner_radius > 0.0
        || !physical_edges_aligned(frame);
    let raster_frame = if analytic {
        padded_raster_frame(frame)
    } else {
        frame
    };
    let x0 = pixel_to_ndc_x(raster_frame.x, viewport.width);
    let x1 = pixel_to_ndc_x(raster_frame.right(), viewport.width);
    let y0 = pixel_to_ndc_y(raster_frame.y, viewport.height);
    let y1 = pixel_to_ndc_y(raster_frame.bottom(), viewport.height);
    let radius = clamped_corner_radius(frame, corner_radius);
    let half_extent = [frame.width * 0.5, frame.height * 0.5];
    let center = [frame.x + half_extent[0], frame.y + half_extent[1]];
    let local = [
        [raster_frame.x - center[0], raster_frame.y - center[1]],
        [raster_frame.right() - center[0], raster_frame.y - center[1]],
        [
            raster_frame.right() - center[0],
            raster_frame.bottom() - center[1],
        ],
        [
            raster_frame.x - center[0],
            raster_frame.bottom() - center[1],
        ],
    ];
    let vertex = |position: [f32; 2], local_position: [f32; 2]| ScreenSpaceUiVertex {
        position,
        color,
        local_position,
        half_extent,
        corner_radius: radius,
        border_width: 0.0,
        fill_color: [0.0; 4],
    };

    vertices.extend_from_slice(&[
        vertex([x0, y0], local[0]),
        vertex([x1, y0], local[1]),
        vertex([x1, y1], local[2]),
        vertex([x0, y0], local[0]),
        vertex([x1, y1], local[2]),
        vertex([x0, y1], local[3]),
    ]);
}

pub(super) fn push_rounded_box(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    fill_color: [f32; 4],
    border_color: [f32; 4],
    border_width: f32,
    corner_radius: f32,
    viewport: UiFrame,
) {
    let start = vertices.len();
    push_rect_geometry(vertices, frame, border_color, corner_radius, viewport, true);
    let width = border_width
        .max(0.0)
        .min(frame.width.min(frame.height).max(0.0) * 0.5);
    for vertex in &mut vertices[start..] {
        vertex.border_width = width;
        vertex.fill_color = fill_color;
    }
}

pub(super) fn push_border(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    border_width: f32,
    color: [f32; 4],
    viewport: UiFrame,
) {
    push_border_with_radius(vertices, frame, border_width, color, 0.0, viewport);
}

pub(super) fn push_border_with_radius(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    frame: UiFrame,
    border_width: f32,
    color: [f32; 4],
    corner_radius: f32,
    viewport: UiFrame,
) {
    if corner_radius.is_finite() && corner_radius > 0.0
        || !physical_edges_aligned(frame)
        || (border_width - border_width.round()).abs() > f32::EPSILON
    {
        let start = vertices.len();
        push_rect_geometry(vertices, frame, color, corner_radius, viewport, true);
        // The fragment shader subtracts the inset signed distance from this one quad.
        let width = border_width
            .max(0.0)
            .min(frame.width.min(frame.height).max(0.0) * 0.5);
        for vertex in &mut vertices[start..] {
            vertex.border_width = width;
        }
        return;
    }
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

pub(super) fn coverage_frame(frame: UiFrame, corner_radius: f32, border_width: f32) -> UiFrame {
    if needs_analytic_coverage(frame, corner_radius, border_width) {
        padded_raster_frame(frame)
    } else {
        frame
    }
}

fn needs_analytic_coverage(frame: UiFrame, corner_radius: f32, border_width: f32) -> bool {
    corner_radius.is_finite() && corner_radius > 0.0
        || !physical_edges_aligned(frame)
        || border_width > 0.0 && (border_width - border_width.round()).abs() > f32::EPSILON
}

fn physical_edges_aligned(frame: UiFrame) -> bool {
    [frame.x, frame.y, frame.right(), frame.bottom()]
        .into_iter()
        .all(|edge| edge.is_finite() && (edge - edge.round()).abs() <= f32::EPSILON)
}

fn padded_raster_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x - 1.0,
        frame.y - 1.0,
        frame.width + 2.0,
        frame.height + 2.0,
    )
}

fn clamped_corner_radius(frame: UiFrame, corner_radius: f32) -> f32 {
    if !corner_radius.is_finite() {
        return 0.0;
    }
    corner_radius
        .max(0.0)
        .min(frame.width.min(frame.height).max(0.0) * 0.5)
}

pub(in crate::graphics::scene::scene_renderer::ui) fn frame_to_scissor(
    frame: UiFrame,
) -> Option<ScreenSpaceUiScissor> {
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

pub(in crate::graphics::scene::scene_renderer::ui) fn clipped_scissor(
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: UiFrame,
    fallback: ScreenSpaceUiScissor,
) -> Option<ScreenSpaceUiScissor> {
    let visible_frame = viewport.intersection(frame)?;
    match clip_frame {
        Some(clip) => visible_frame.intersection(clip).and_then(frame_to_scissor),
        None => Some(fallback),
    }
}

fn pixel_to_ndc_x(x: f32, width: f32) -> f32 {
    (x / width.max(1.0)) * 2.0 - 1.0
}

fn pixel_to_ndc_y(y: f32, height: f32) -> f32 {
    1.0 - (y / height.max(1.0)) * 2.0
}
