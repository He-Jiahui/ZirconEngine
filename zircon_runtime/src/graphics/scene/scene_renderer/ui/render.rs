use std::ops::Range;

use wgpu::util::DeviceExt;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiPaintPayload, UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiResolvedStyle,
    UiTextAlign, UiTextDirection, UiTextPaintDecorationKind, UiTextRange, UiTextRenderMode,
    UiTextRunPaintStyle, UiTextWrap, UiTextWritingMode,
};

use crate::core::framework::render::SkyboxMode;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

use super::screen_space_ui_renderer::ScreenSpaceUiRenderer;

mod background;
mod color;
mod geometry;

use background::{text_batch_background_color, ScreenSpaceUiBackgroundTracker};
use color::parse_color;
pub(super) use geometry::ScreenSpaceUiVertex;
use geometry::{frame_to_scissor, push_rect, ScreenSpaceUiScissor};

struct PreparedScreenSpaceUi {
    vertex_buffer: Option<wgpu::Buffer>,
    draws: Vec<ScreenSpaceUiDraw>,
    post_text_draws: Vec<ScreenSpaceUiDraw>,
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
    pub(super) source_range: Option<UiTextRange>,
    pub(super) glyph_advances: Vec<f32>,
    pub(super) color: [f32; 4],
    pub(super) background_color: Option<[f32; 4]>,
    pub(super) font: Option<String>,
    pub(super) font_family: Option<String>,
    pub(super) font_weight: u16,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) text_align: UiTextAlign,
    pub(super) text_direction: UiTextDirection,
    pub(super) writing_mode: UiTextWritingMode,
    pub(super) wrap: UiTextWrap,
    pub(super) style: UiTextRunPaintStyle,
}

impl ScreenSpaceUiRenderer {
    pub(crate) fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let pass_clear_color = wgpu::Color::TRANSPARENT;
        let Some(prepared) =
            prepare_screen_space_ui(device, frame, attachment_ops, pass_clear_color)
        else {
            self.last_text_prepare_report = Default::default();
            return;
        };
        self.last_attachment_ops = attachment_ops;
        self.text_system.prepare(
            device,
            queue,
            frame.viewport_size,
            &prepared.auto_texts,
            &prepared.native_texts,
            &prepared.sdf_texts,
        );
        self.last_text_prepare_report = self.text_system.prepare_report();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-screen-space-ui-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: color_attachment_operations(attachment_ops, pass_clear_color),
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

        if !prepared.post_text_draws.is_empty() {
            pass.set_pipeline(&self.pipeline);
            if let Some(vertex_buffer) = prepared.vertex_buffer.as_ref() {
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            }
            for draw in &prepared.post_text_draws {
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

    pub(crate) fn text_prepare_report(&self) -> super::text::ScreenSpaceUiTextPrepareReport {
        self.last_text_prepare_report.clone()
    }

    #[cfg(test)]
    pub(crate) fn last_attachment_ops(&self) -> RenderGraphAttachmentOps {
        self.last_attachment_ops
    }
}

fn prepare_screen_space_ui(
    device: &wgpu::Device,
    frame: &ViewportRenderFrame,
    attachment_ops: RenderGraphAttachmentOps,
    pass_clear_color: wgpu::Color,
) -> Option<PreparedScreenSpaceUi> {
    let extract = frame.ui.as_ref()?;
    let framebuffer_background_color =
        framebuffer_background_color(frame, attachment_ops, pass_clear_color);
    let plan = plan_screen_space_ui_batches_with_framebuffer_background(
        extract,
        frame.viewport_size,
        framebuffer_background_color,
    );

    if plan.draws.is_empty()
        && plan.post_text_draws.is_empty()
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
        post_text_draws: plan.post_text_draws,
        auto_texts: plan.auto_texts,
        native_texts: plan.native_texts,
        sdf_texts: plan.sdf_texts,
    })
}

struct PlannedScreenSpaceUi {
    vertices: Vec<ScreenSpaceUiVertex>,
    draws: Vec<ScreenSpaceUiDraw>,
    post_text_draws: Vec<ScreenSpaceUiDraw>,
    auto_texts: Vec<ScreenSpaceUiTextBatch>,
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
}

fn plan_screen_space_ui_batches(
    extract: &UiRenderExtract,
    viewport_size: crate::core::math::UVec2,
) -> PlannedScreenSpaceUi {
    plan_screen_space_ui_batches_with_framebuffer_background(extract, viewport_size, None)
}

fn plan_screen_space_ui_batches_with_framebuffer_background(
    extract: &UiRenderExtract,
    viewport_size: crate::core::math::UVec2,
    framebuffer_background_color: Option<[f32; 4]>,
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
        post_text_draws: Vec::new(),
        auto_texts: Vec::new(),
        native_texts: Vec::new(),
        sdf_texts: Vec::new(),
    };
    let mut backgrounds = ScreenSpaceUiBackgroundTracker::with_framebuffer_background(
        viewport,
        framebuffer_background_color,
    );

    for command in &extract.list.commands {
        let scissor = command_scissor(command, viewport, full_scissor);
        let start = plan.vertices.len() as u32;
        plan_command_batches(command, viewport, &backgrounds, &mut plan);
        let end = plan.vertices.len() as u32;
        if end > start {
            plan.draws.push(ScreenSpaceUiDraw {
                vertices: start..end,
                scissor,
            });
        }

        let post_text_start = plan.vertices.len() as u32;
        push_text_decoration_vertices(command, viewport, &mut plan.vertices, false);
        let post_text_end = plan.vertices.len() as u32;
        if post_text_end > post_text_start {
            plan.post_text_draws.push(ScreenSpaceUiDraw {
                vertices: post_text_start..post_text_end,
                scissor,
            });
        }
        backgrounds.observe_command(command, viewport);
    }

    plan
}

fn framebuffer_background_color(
    frame: &ViewportRenderFrame,
    attachment_ops: RenderGraphAttachmentOps,
    pass_clear_color: wgpu::Color,
) -> Option<[f32; 4]> {
    match attachment_ops.load {
        RenderGraphAttachmentLoadOp::Clear => opaque_wgpu_color(pass_clear_color),
        RenderGraphAttachmentLoadOp::Load => known_loaded_framebuffer_background_color(frame),
    }
}

fn known_loaded_framebuffer_background_color(frame: &ViewportRenderFrame) -> Option<[f32; 4]> {
    let overlays = frame.overlays();
    let has_overlay_content = !overlays.selection.is_empty()
        || !overlays.selection_anchors.is_empty()
        || overlays.grid.as_ref().is_some_and(|grid| grid.visible)
        || !overlays.handles.is_empty()
        || !overlays.scene_gizmos.is_empty();
    if frame.environment().skybox.mode != SkyboxMode::Disabled
        || frame.preview().skybox_enabled
        || !frame.meshes().is_empty()
        || !frame.sprites().is_empty()
        || loaded_frame_has_particle_content(frame)
        || has_overlay_content
    {
        return None;
    }

    let clear = frame.preview().clear_color;
    opaque_f32_color([clear.x, clear.y, clear.z, clear.w])
}

fn loaded_frame_has_particle_content(frame: &ViewportRenderFrame) -> bool {
    let particles = &frame.extract.particles;
    !particles.emitters.is_empty()
        || !particles.sprites.is_empty()
        || !particles.previous_sprites.is_empty()
        || !particles.bounds.is_empty()
        || particles
            .gpu_frame
            .as_ref()
            .is_some_and(|gpu| gpu.alive_count > 0 || gpu.spawned_total > 0)
}

fn opaque_wgpu_color(color: wgpu::Color) -> Option<[f32; 4]> {
    opaque_f32_color([
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    ])
}

fn opaque_f32_color(color: [f32; 4]) -> Option<[f32; 4]> {
    if color.iter().all(|component| component.is_finite()) && color[3] >= 1.0 {
        Some([
            color[0].clamp(0.0, 1.0),
            color[1].clamp(0.0, 1.0),
            color[2].clamp(0.0, 1.0),
            1.0,
        ])
    } else {
        None
    }
}

fn command_scissor(
    command: &UiRenderCommand,
    viewport: UiFrame,
    fallback: ScreenSpaceUiScissor,
) -> ScreenSpaceUiScissor {
    command
        .clip_frame
        .and_then(|clip| viewport.intersection(clip))
        .and_then(frame_to_scissor)
        .unwrap_or(fallback)
}

fn plan_command_batches(
    command: &UiRenderCommand,
    viewport: UiFrame,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
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
        push_text_decoration_vertices(command, viewport, &mut plan.vertices, true);
        push_text_batches(command, frame, color, viewport, backgrounds, plan);
    }
}

fn push_text_decoration_vertices(
    command: &UiRenderCommand,
    viewport: UiFrame,
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    before_text: bool,
) {
    for element in command.to_paint_elements(0) {
        let UiPaintPayload::Text { text } = element.payload else {
            continue;
        };
        for decoration in text.decorations {
            let decoration_before_text =
                matches!(decoration.kind, UiTextPaintDecorationKind::Selection);
            if decoration_before_text != before_text {
                continue;
            }
            let Some(frame) = viewport.intersection(decoration.frame) else {
                continue;
            };
            let color = parse_color(
                Some(decoration.color.as_str()),
                text_decoration_fallback_color(decoration.kind),
                command.opacity,
            )
            .unwrap_or_else(|| text_decoration_fallback_color(decoration.kind));
            push_rect(vertices, frame, color, viewport);
        }
    }
}

fn text_decoration_fallback_color(kind: UiTextPaintDecorationKind) -> [f32; 4] {
    match kind {
        UiTextPaintDecorationKind::Selection => [0.30, 0.54, 1.0, 0.40],
        UiTextPaintDecorationKind::CompositionUnderline => [0.30, 0.54, 1.0, 1.0],
        UiTextPaintDecorationKind::Caret => [0.91, 0.93, 0.97, 1.0],
        UiTextPaintDecorationKind::Outline => [0.91, 0.93, 0.97, 1.0],
    }
}

fn push_text_batches(
    command: &UiRenderCommand,
    fallback_frame: UiFrame,
    color: [f32; 4],
    viewport: UiFrame,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    if let Some(layout) = command
        .text_layout
        .as_ref()
        .filter(|layout| !layout.lines.is_empty())
    {
        if !command.style.rich_text {
            push_resolved_text_layout_line_batches(
                command,
                layout,
                color,
                viewport,
                backgrounds,
                plan,
            );
            return;
        }
    }

    if let Some(text_paint) = command_text_paint(command) {
        if !text_paint.runs.is_empty() {
            for run in text_paint.runs {
                let font = run.font.or_else(|| command.style.font.clone());
                let font_family = run
                    .font_family
                    .or_else(|| command.style.font_family.clone());
                let run_color =
                    parse_color(run.color.as_deref(), color, command.opacity).unwrap_or(color);
                push_text_batch(
                    command,
                    run.text,
                    run.frame,
                    Some(run.source_range),
                    Vec::new(),
                    font,
                    font_family,
                    run.font_weight,
                    run.font_size,
                    run.line_height,
                    run_color,
                    UiTextAlign::Left,
                    command.style.text_direction,
                    text_paint.writing_mode,
                    UiTextWrap::None,
                    run.style,
                    viewport,
                    backgrounds,
                    plan,
                );
            }
            return;
        }
    }

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
                Some(line.source_range),
                line.glyph_advances.clone(),
                command.style.font.clone(),
                command.style.font_family.clone(),
                command.style.font_weight,
                layout.font_size,
                layout.line_height,
                color,
                command.style.text_align,
                line.direction,
                layout.writing_mode,
                command.style.wrap,
                UiTextRunPaintStyle::default(),
                viewport,
                backgrounds,
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
            None,
            Vec::new(),
            command.style.font.clone(),
            command.style.font_family.clone(),
            command.style.font_weight,
            font_size,
            command.style.line_height.max(font_size),
            color,
            command.style.text_align,
            command.style.text_direction,
            command.style.text_writing_mode,
            command.style.wrap,
            UiTextRunPaintStyle::default(),
            viewport,
            backgrounds,
            plan,
        );
    }
}

fn push_resolved_text_layout_line_batches(
    command: &UiRenderCommand,
    layout: &zircon_runtime_interface::ui::surface::UiResolvedTextLayout,
    color: [f32; 4],
    viewport: UiFrame,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    for line in &layout.lines {
        push_text_batch(
            command,
            line.text.clone(),
            line.frame,
            Some(line.source_range),
            line.glyph_advances.clone(),
            command.style.font.clone(),
            command.style.font_family.clone(),
            command.style.font_weight,
            layout.font_size,
            layout.line_height,
            color,
            UiTextAlign::Left,
            line.direction,
            layout.writing_mode,
            UiTextWrap::None,
            UiTextRunPaintStyle::default(),
            viewport,
            backgrounds,
            plan,
        );
    }
}

fn command_text_paint(
    command: &UiRenderCommand,
) -> Option<zircon_runtime_interface::ui::surface::UiTextPaint> {
    command
        .to_paint_elements(0)
        .into_iter()
        .find_map(|element| match element.payload {
            UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
}

fn push_text_batch(
    command: &UiRenderCommand,
    text: String,
    frame: UiFrame,
    source_range: Option<UiTextRange>,
    glyph_advances: Vec<f32>,
    font: Option<String>,
    font_family: Option<String>,
    font_weight: u16,
    font_size: f32,
    line_height: f32,
    color: [f32; 4],
    text_align: UiTextAlign,
    text_direction: UiTextDirection,
    writing_mode: UiTextWritingMode,
    wrap: UiTextWrap,
    style: UiTextRunPaintStyle,
    viewport: UiFrame,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    if text.is_empty() || frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    let batch = ScreenSpaceUiTextBatch {
        text,
        frame,
        clip_frame: command.clip_frame,
        source_range,
        glyph_advances,
        color,
        background_color: text_batch_background_color(command, frame, viewport, backgrounds),
        font,
        font_family,
        font_weight: UiResolvedStyle::normalized_font_weight(font_weight),
        font_size: font_size.max(1.0),
        line_height: line_height.max(font_size.max(1.0)),
        text_align,
        text_direction,
        writing_mode,
        wrap,
        style,
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

#[cfg(test)]
mod tests;
