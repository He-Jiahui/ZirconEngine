use std::ops::Range;

use wgpu::util::DeviceExt;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    normalize_ui_text_language_tag, UiPaintPayload, UiRenderCommand, UiRenderCommandKind,
    UiRenderExtract, UiResolvedStyle, UiTextAlign, UiTextDecorations, UiTextDirection,
    UiTextPaintDecorationKind, UiTextRange, UiTextRenderMode, UiTextRunPaintStyle, UiTextWrap,
    UiTextWritingMode,
};

use crate::core::framework::render::SkyboxMode;
use crate::core::framework::text::TextLayoutError;
use crate::graphics::scene::resources::{ui_image_resource_id, ResourceStreamer};
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};
use crate::text::sdf::SdfMode;

use super::image::ScreenSpaceUiImageBatch;
use super::screen_space_ui_renderer::ScreenSpaceUiRenderer;

mod background;
mod color;
mod geometry;
mod rich_text;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_advances;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_decorations;
mod text_distance_field;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_effects;
mod text_paint;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_projection;

use background::{text_batch_background_color, ScreenSpaceUiBackgroundTracker};
use color::parse_color;
pub(super) use geometry::ScreenSpaceUiVertex;
pub(super) use geometry::{frame_to_scissor, ScreenSpaceUiScissor};
use geometry::{push_border, push_rect};
pub(super) use text_advances::ScreenSpaceUiShapedGlyph;
use text_decorations::{
    resolve_text_decorations, resolved_text_decoration_baseline, ScreenSpaceUiTextDecorations,
};
use text_distance_field::resolved_text_distance_field_mode;
use text_effects::{resolve_text_effects, ScreenSpaceUiTextEffects};

struct PreparedScreenSpaceUi {
    vertex_buffer: Option<wgpu::Buffer>,
    draws: Vec<ScreenSpaceUiDraw>,
    post_text_draws: Vec<ScreenSpaceUiDraw>,
    auto_texts: Vec<ScreenSpaceUiTextBatch>,
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
    images: Vec<ScreenSpaceUiImageBatch>,
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
    pub(super) shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
    pub(super) layout_error: Option<TextLayoutError>,
    pub(super) color: [f32; 4],
    pub(super) background_color: Option<[f32; 4]>,
    pub(super) font: Option<String>,
    pub(super) font_family: Option<String>,
    pub(super) language: Option<String>,
    pub(super) font_weight: u16,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) text_align: UiTextAlign,
    pub(super) text_direction: UiTextDirection,
    pub(super) writing_mode: UiTextWritingMode,
    pub(super) wrap: UiTextWrap,
    pub(super) style: UiTextRunPaintStyle,
    pub(super) distance_field_mode: SdfMode,
    pub(super) text_effects: ScreenSpaceUiTextEffects,
    pub(super) text_decorations: ScreenSpaceUiTextDecorations,
    pub(super) text_decoration_baseline: Option<f32>,
    pub(super) clip_transform: Option<text_projection::ScreenSpaceUiTextClipTransform>,
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
        streamer: Option<&ResourceStreamer>,
    ) -> Result<(), GraphicsError> {
        let pass_clear_color = wgpu::Color::TRANSPARENT;
        let Some(prepared) =
            prepare_screen_space_ui(device, frame, attachment_ops, pass_clear_color)
        else {
            self.last_text_prepare_report = Default::default();
            return Ok(());
        };
        self.last_attachment_ops = attachment_ops;
        self.text_system
            .prepare(
                device,
                queue,
                frame.viewport_size,
                &prepared.auto_texts,
                &prepared.native_texts,
                &prepared.sdf_texts,
            )
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        self.last_text_prepare_report = self.text_system.prepare_report();
        let prepared_images =
            self.image_system
                .prepare(device, frame.viewport_size, &prepared.images, streamer);

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
        self.image_system.render(&mut pass, &prepared_images);
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
        Ok(())
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
        && plan.images.is_empty()
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
        images: plan.images,
    })
}

struct PlannedScreenSpaceUi {
    vertices: Vec<ScreenSpaceUiVertex>,
    draws: Vec<ScreenSpaceUiDraw>,
    post_text_draws: Vec<ScreenSpaceUiDraw>,
    auto_texts: Vec<ScreenSpaceUiTextBatch>,
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
    images: Vec<ScreenSpaceUiImageBatch>,
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
        images: Vec::new(),
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

    if let Some(zircon_runtime_interface::ui::surface::UiVisualAssetRef::Image(source)) =
        command.image.as_ref()
    {
        if let Some(texture) = ui_image_resource_id(source) {
            plan.images.push(ScreenSpaceUiImageBatch {
                texture,
                frame: command.frame,
                clip_frame: command.clip_frame,
                tint: [1.0, 1.0, 1.0, command.opacity.clamp(0.0, 1.0)],
            });
        }
    } else if command.image.is_some() || matches!(command.kind, UiRenderCommandKind::Image) {
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
            let decoration_before_text = matches!(
                decoration.kind,
                UiTextPaintDecorationKind::Selection
                    | UiTextPaintDecorationKind::TableCellBackground
            );
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
            if matches!(decoration.kind, UiTextPaintDecorationKind::TableCellBorder) {
                push_border(vertices, frame, decoration.thickness, color, viewport);
            } else {
                push_rect(vertices, frame, color, viewport);
            }
        }
    }
}

fn text_decoration_fallback_color(kind: UiTextPaintDecorationKind) -> [f32; 4] {
    match kind {
        UiTextPaintDecorationKind::Selection => [0.30, 0.54, 1.0, 0.40],
        UiTextPaintDecorationKind::CompositionUnderline => [0.30, 0.54, 1.0, 1.0],
        UiTextPaintDecorationKind::Caret => [0.91, 0.93, 0.97, 1.0],
        UiTextPaintDecorationKind::Outline => [0.91, 0.93, 0.97, 1.0],
        UiTextPaintDecorationKind::TableCellBackground => [0.0, 0.0, 0.0, 0.0],
        UiTextPaintDecorationKind::TableCellBorder => [0.91, 0.93, 0.97, 1.0],
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
        if matches!(
            command.style.rich_text_format,
            zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
        ) {
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

    if let Some(text_paint) = text_paint::command_text_paint(command) {
        if !text_paint.runs.is_empty() {
            let parsed_rich = rich_text::parse_command_rich_text(command);
            for run in text_paint.runs {
                let rich_run = parsed_rich
                    .as_ref()
                    .and_then(|parsed| rich_text::run_for_range(parsed, run.source_range));
                if rich_text::plan_inline_run(
                    command,
                    &run,
                    rich_run,
                    viewport,
                    color,
                    backgrounds,
                    plan,
                ) {
                    continue;
                }
                let presentation =
                    rich_text::prepare_text_run(command, &run, rich_run, viewport, color, plan);
                push_text_batch(
                    command,
                    run.text,
                    run.frame,
                    Some(run.source_range),
                    Vec::new(),
                    presentation.font,
                    presentation.font_family,
                    presentation.font_weight,
                    presentation.font_size,
                    presentation.line_height,
                    presentation.color,
                    UiTextAlign::Left,
                    command.style.text_direction,
                    text_paint.writing_mode,
                    UiTextWrap::None,
                    run.style,
                    presentation.text_decorations,
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
                command.style.text_decorations.clone(),
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
            command.style.text_decorations.clone(),
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
            command.style.text_decorations.clone(),
            viewport,
            backgrounds,
            plan,
        );
    }
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
    text_decorations: UiTextDecorations,
    viewport: UiFrame,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    if text.is_empty() || frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    let language = normalize_ui_text_language_tag(command.style.language.as_deref());
    let resolved_source_range = source_range.unwrap_or(UiTextRange {
        start: 0,
        end: text.len(),
    });
    let resolved_glyphs = text_advances::resolve_screen_space_text_glyphs(
        text_advances::ScreenSpaceTextShapingRequest {
            text: text.as_str(),
            font: font.as_deref(),
            font_family: font_family.as_deref(),
            language: language.as_deref(),
            font_weight,
            font_size,
            line_height,
            direction: text_direction,
            writing_mode,
            source_range: resolved_source_range,
        },
        glyph_advances,
    );
    let text_advances::ResolvedScreenSpaceTextGlyphs {
        glyph_advances,
        shaped_glyphs,
        layout_error,
    } = resolved_glyphs;

    let text_effects = command.style.text_effects.normalized();
    let distance_field_mode =
        resolved_text_distance_field_mode(command.style.text_render_mode, font_size, &text_effects);
    let text_decoration_baseline =
        resolved_text_decoration_baseline(command, source_range, writing_mode);
    let text_decorations = resolve_text_decorations(&text_decorations, color, command.opacity);
    let batch = ScreenSpaceUiTextBatch {
        text,
        frame,
        clip_frame: command.clip_frame,
        source_range,
        glyph_advances,
        shaped_glyphs,
        layout_error,
        color,
        background_color: text_batch_background_color(command, frame, viewport, backgrounds),
        font,
        font_family,
        language,
        font_weight: UiResolvedStyle::normalized_font_weight(font_weight),
        font_size: font_size.max(1.0),
        line_height: line_height.max(font_size.max(1.0)),
        text_align,
        text_direction,
        writing_mode,
        wrap,
        style,
        distance_field_mode,
        text_effects: resolve_text_effects(&text_effects, command.opacity),
        text_decorations,
        text_decoration_baseline,
        clip_transform: None,
    };
    match (
        command.style.text_render_mode,
        text_effects.requires_distance_field(),
    ) {
        (UiTextRenderMode::Auto | UiTextRenderMode::Native, true) => plan.sdf_texts.push(batch),
        (UiTextRenderMode::Auto, false) => plan.auto_texts.push(batch),
        (UiTextRenderMode::Native, false) => plan.native_texts.push(batch),
        (UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf, _) => {
            plan.sdf_texts.push(batch)
        }
    }
}

#[cfg(all(test, feature = "ui"))]
mod tests;
