use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiTextAlign, UiTextRenderMode,
    UiTextWrap,
};

use crate::graphics::types::ViewportRenderFrame;

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
    vertex_buffer: Option<wgpu::Buffer>,
    draws: Vec<ScreenSpaceUiDraw>,
    auto_texts: Vec<ScreenSpaceUiTextBatch>,
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
}

struct ScreenSpaceUiDraw {
    vertices: Range<u32>,
    scissor: ScreenSpaceUiScissor,
}

#[derive(Clone, Debug)]
pub(super) struct ScreenSpaceUiTextBatch {
    pub(super) text: String,
    pub(super) frame: UiFrame,
    pub(super) clip_frame: Option<UiFrame>,
    pub(super) color: [f32; 4],
    pub(super) font: Option<String>,
    pub(super) font_family: Option<String>,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) text_align: UiTextAlign,
    pub(super) wrap: UiTextWrap,
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
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
    ) {
        let Some(prepared) = prepare_screen_space_ui(device, frame) else {
            return;
        };
        self.text_system.prepare(
            device,
            queue,
            frame.viewport_size,
            &prepared.auto_texts,
            &prepared.native_texts,
            &prepared.sdf_texts,
        );

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
        if let Some(vertex_buffer) = prepared.vertex_buffer.as_ref() {
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }

        for draw in &prepared.draws {
            pass.set_scissor_rect(
                draw.scissor.x,
                draw.scissor.y,
                draw.scissor.width,
                draw.scissor.height,
            );
            pass.draw(draw.vertices.clone(), 0..1);
        }
        pass.set_scissor_rect(
            0,
            0,
            frame.viewport_size.x.max(1),
            frame.viewport_size.y.max(1),
        );
        self.text_system.render(&mut pass);
    }
}

fn prepare_screen_space_ui(
    device: &wgpu::Device,
    frame: &ViewportRenderFrame,
) -> Option<PreparedScreenSpaceUi> {
    let extract = frame.ui.as_ref()?;
    let plan = plan_screen_space_ui_batches(extract, frame.viewport_size);

    if plan.draws.is_empty()
        && plan.auto_texts.is_empty()
        && plan.native_texts.is_empty()
        && plan.sdf_texts.is_empty()
    {
        return None;
    }

    let vertex_buffer = (!plan.vertices.is_empty()).then(|| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-screen-space-ui-vertices"),
            contents: bytemuck::cast_slice(&plan.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    });

    Some(PreparedScreenSpaceUi {
        vertex_buffer,
        draws: plan.draws,
        auto_texts: plan.auto_texts,
        native_texts: plan.native_texts,
        sdf_texts: plan.sdf_texts,
    })
}

struct PlannedScreenSpaceUi {
    vertices: Vec<ScreenSpaceUiVertex>,
    draws: Vec<ScreenSpaceUiDraw>,
    auto_texts: Vec<ScreenSpaceUiTextBatch>,
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
}

fn plan_screen_space_ui_batches(
    extract: &UiRenderExtract,
    viewport_size: crate::core::math::UVec2,
) -> PlannedScreenSpaceUi {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let full_scissor = ScreenSpaceUiScissor {
        x: 0,
        y: 0,
        width: viewport_size.x.max(1),
        height: viewport_size.y.max(1),
    };

    let mut plan = PlannedScreenSpaceUi {
        vertices: Vec::new(),
        draws: Vec::new(),
        auto_texts: Vec::new(),
        native_texts: Vec::new(),
        sdf_texts: Vec::new(),
    };

    for command in &extract.list.commands {
        let start = plan.vertices.len() as u32;
        plan_command_batches(command, viewport, &mut plan);
        let end = plan.vertices.len() as u32;
        if end > start {
            let scissor = command
                .clip_frame
                .and_then(|clip| viewport.intersection(clip))
                .and_then(frame_to_scissor)
                .unwrap_or(full_scissor);
            plan.draws.push(ScreenSpaceUiDraw {
                vertices: start..end,
                scissor,
            });
        }
    }

    plan
}

fn plan_command_batches(
    command: &UiRenderCommand,
    viewport: UiFrame,
    plan: &mut PlannedScreenSpaceUi,
) {
    if command.opacity <= 0.0 {
        return;
    }

    let frame = match viewport.intersection(command.frame) {
        Some(frame) => frame,
        None => return,
    };

    if matches!(command.kind, UiRenderCommandKind::Quad)
        || command.style.background_color.is_some()
        || command.style.border_color.is_some()
        || command.style.border_width > 0.0
    {
        if let Some(color) = parse_color(
            command.style.background_color.as_deref(),
            [0.16, 0.19, 0.24, 1.0],
            command.opacity,
        ) {
            push_rect(&mut plan.vertices, frame, color, viewport);
        }
        let border_width = command.style.border_width.max(0.0);
        if border_width > 0.0 {
            let color = parse_color(
                command.style.border_color.as_deref(),
                [0.85, 0.88, 0.92, 1.0],
                command.opacity,
            )
            .unwrap_or([0.85, 0.88, 0.92, command.opacity]);
            push_border(&mut plan.vertices, frame, border_width, color, viewport);
        }
    }

    if command.image.is_some() || matches!(command.kind, UiRenderCommandKind::Image) {
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
        push_rect(&mut plan.vertices, icon, color, viewport);
    }

    if command.text.as_ref().is_some_and(|text| !text.is_empty()) {
        let color = parse_color(
            command.style.foreground_color.as_deref(),
            [0.96, 0.96, 0.96, 1.0],
            command.opacity,
        )
        .unwrap_or([0.96, 0.96, 0.96, command.opacity]);
        push_text_batches(command, frame, color, plan);
    }
}

fn push_text_batches(
    command: &UiRenderCommand,
    fallback_frame: UiFrame,
    color: [f32; 4],
    plan: &mut PlannedScreenSpaceUi,
) {
    if let Some(layout) = command
        .text_layout
        .as_ref()
        .filter(|layout| !layout.lines.is_empty())
    {
        for line in &layout.lines {
            push_text_batch(
                command,
                line.text.clone(),
                line.frame,
                layout.font_size,
                layout.line_height,
                color,
                plan,
            );
        }
        return;
    }

    if let Some(text) = command.text.as_ref().filter(|text| !text.is_empty()) {
        let font_size = command.style.font_size.max(1.0);
        push_text_batch(
            command,
            text.clone(),
            fallback_frame,
            font_size,
            command.style.line_height.max(font_size),
            color,
            plan,
        );
    }
}

fn push_text_batch(
    command: &UiRenderCommand,
    text: String,
    frame: UiFrame,
    font_size: f32,
    line_height: f32,
    color: [f32; 4],
    plan: &mut PlannedScreenSpaceUi,
) {
    if text.is_empty() || frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    let batch = ScreenSpaceUiTextBatch {
        text,
        frame,
        clip_frame: command.clip_frame,
        color,
        font: command.style.font.clone(),
        font_family: command.style.font_family.clone(),
        font_size: font_size.max(1.0),
        line_height: line_height.max(font_size.max(1.0)),
        text_align: command.style.text_align,
        wrap: command.style.wrap,
    };
    match command.style.text_render_mode {
        UiTextRenderMode::Auto => plan.auto_texts.push(batch),
        UiTextRenderMode::Native => plan.native_texts.push(batch),
        UiTextRenderMode::Sdf => plan.sdf_texts.push(batch),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::UVec2;
    use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
    use zircon_runtime_interface::ui::surface::{
        UiRenderExtract, UiRenderList, UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine,
        UiResolvedTextRun, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRange,
        UiTextRenderMode, UiTextRunKind, UiTextWrap,
    };

    #[test]
    fn screen_space_ui_plan_keeps_text_batches_for_quad_commands() {
        let plan = plan_screen_space_ui_batches(
            &UiRenderExtract {
                tree_id: UiTreeId::new("runtime.ui"),
                list: UiRenderList {
                    commands: vec![UiRenderCommand {
                        node_id: UiNodeId::new(1),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            background_color: Some("#112233".to_string()),
                            foreground_color: Some("#ddeeff".to_string()),
                            font_size: 18.0,
                            line_height: 22.0,
                            text_align: UiTextAlign::Center,
                            wrap: UiTextWrap::Word,
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("Launch".to_string()),
                        image: None,
                        opacity: 1.0,
                    }],
                },
            },
            UVec2::new(200, 100),
        );

        assert_eq!(plan.draws.len(), 1);
        assert_eq!(plan.native_texts.len(), 1);
        assert!(plan.sdf_texts.is_empty());
    }

    #[test]
    fn screen_space_ui_plan_routes_sdf_text_to_a_separate_batch() {
        let plan = plan_screen_space_ui_batches(
            &UiRenderExtract {
                tree_id: UiTreeId::new("runtime.ui"),
                list: UiRenderList {
                    commands: vec![UiRenderCommand {
                        node_id: UiNodeId::new(2),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(0.0, 0.0, 160.0, 48.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ffffff".to_string()),
                            font_size: 20.0,
                            line_height: 24.0,
                            text_align: UiTextAlign::Left,
                            wrap: UiTextWrap::Word,
                            text_render_mode: UiTextRenderMode::Sdf,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("SDF".to_string()),
                        image: None,
                        opacity: 1.0,
                    }],
                },
            },
            UVec2::new(320, 180),
        );

        assert!(plan.native_texts.is_empty());
        assert_eq!(plan.sdf_texts.len(), 1);
    }

    #[test]
    fn screen_space_ui_plan_keeps_auto_text_in_a_separate_batch() {
        let plan = plan_screen_space_ui_batches(
            &UiRenderExtract {
                tree_id: UiTreeId::new("runtime.ui"),
                list: UiRenderList {
                    commands: vec![UiRenderCommand {
                        node_id: UiNodeId::new(3),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(4.0, 6.0, 144.0, 40.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ffffff".to_string()),
                            font: Some("res://fonts/default.font.toml".to_string()),
                            font_size: 16.0,
                            line_height: 20.0,
                            text_align: UiTextAlign::Left,
                            wrap: UiTextWrap::Word,
                            text_render_mode: UiTextRenderMode::Auto,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("Auto".to_string()),
                        image: None,
                        opacity: 1.0,
                    }],
                },
            },
            UVec2::new(320, 180),
        );

        assert!(plan.native_texts.is_empty());
        assert!(plan.sdf_texts.is_empty());
        assert_eq!(plan.auto_texts.len(), 1);
    }

    #[test]
    fn screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches() {
        let plan = plan_screen_space_ui_batches(
            &UiRenderExtract {
                tree_id: UiTreeId::new("runtime.ui"),
                list: UiRenderList {
                    commands: vec![UiRenderCommand {
                        node_id: UiNodeId::new(4),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(10.0, 20.0, 90.0, 48.0),
                        clip_frame: Some(UiFrame::new(0.0, 0.0, 120.0, 48.0)),
                        z_index: 0,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ffffff".to_string()),
                            font_size: 10.0,
                            line_height: 12.0,
                            text_align: UiTextAlign::Center,
                            wrap: UiTextWrap::Word,
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: Some(UiResolvedTextLayout {
                            text_align: UiTextAlign::Center,
                            wrap: UiTextWrap::Word,
                            direction: UiTextDirection::LeftToRight,
                            overflow: UiTextOverflow::Clip,
                            font_size: 10.0,
                            line_height: 12.0,
                            measured_width: 50.0,
                            measured_height: 24.0,
                            source_range: UiTextRange { start: 0, end: 16 },
                            lines: vec![
                                UiResolvedTextLine {
                                    text: "Alpha Beta".to_string(),
                                    frame: UiFrame::new(20.0, 20.0, 50.0, 12.0),
                                    source_range: UiTextRange { start: 0, end: 10 },
                                    visual_range: UiTextRange { start: 0, end: 10 },
                                    measured_width: 50.0,
                                    baseline: 8.0,
                                    direction: UiTextDirection::LeftToRight,
                                    runs: vec![UiResolvedTextRun {
                                        kind: UiTextRunKind::Plain,
                                        text: "Alpha Beta".to_string(),
                                        source_range: UiTextRange { start: 0, end: 10 },
                                        visual_range: UiTextRange { start: 0, end: 10 },
                                        direction: UiTextDirection::LeftToRight,
                                    }],
                                    ellipsized: false,
                                },
                                UiResolvedTextLine {
                                    text: "Gamma".to_string(),
                                    frame: UiFrame::new(35.0, 32.0, 25.0, 12.0),
                                    source_range: UiTextRange { start: 11, end: 16 },
                                    visual_range: UiTextRange { start: 0, end: 5 },
                                    measured_width: 25.0,
                                    baseline: 8.0,
                                    direction: UiTextDirection::LeftToRight,
                                    runs: vec![UiResolvedTextRun {
                                        kind: UiTextRunKind::Plain,
                                        text: "Gamma".to_string(),
                                        source_range: UiTextRange { start: 11, end: 16 },
                                        visual_range: UiTextRange { start: 0, end: 5 },
                                        direction: UiTextDirection::LeftToRight,
                                    }],
                                    ellipsized: false,
                                },
                            ],
                            overflow_clipped: false,
                            editable: None,
                        }),
                        text: Some("Alpha Beta Gamma".to_string()),
                        image: None,
                        opacity: 1.0,
                    }],
                },
            },
            UVec2::new(160, 120),
        );

        assert_eq!(plan.native_texts.len(), 2);
        assert_eq!(plan.native_texts[0].text, "Alpha Beta");
        assert_eq!(
            plan.native_texts[0].frame,
            UiFrame::new(20.0, 20.0, 50.0, 12.0)
        );
        assert_eq!(plan.native_texts[1].text, "Gamma");
        assert_eq!(
            plan.native_texts[1].frame,
            UiFrame::new(35.0, 32.0, 25.0, 12.0)
        );
        assert_eq!(
            plan.native_texts[0].clip_frame,
            Some(UiFrame::new(0.0, 0.0, 120.0, 48.0))
        );
    }
}
