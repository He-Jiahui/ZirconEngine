use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiPaintElement, UiRenderCommand, UiResolvedStyle, UiTextAlign, UiTextDecorations,
    UiTextDirection, UiTextRange, UiTextRenderMode, UiTextRunPaintStyle, UiTextWrap,
    UiTextWritingMode,
};

use crate::core::framework::text::TextLayoutError;
use crate::text::sdf::SdfMode;
use crate::text::text_language_cache_identity;

use super::background::{text_batch_background_color, ScreenSpaceUiBackgroundTracker};
use super::resolved_layout;
use super::rich_text;
use super::text_decorations::{
    resolve_text_decorations, resolved_text_decoration_baseline, ScreenSpaceUiTextDecorations,
};
use super::text_distance_field::resolved_text_distance_field_mode;
use super::text_effects::{resolve_text_effects, ScreenSpaceUiTextEffects};
use super::text_provenance::is_source_isomorphic_resolved_text_line;
use super::{
    text_paint, PlannedScreenSpaceUi, ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiShapedGlyph,
    ScreenSpaceUiTextRouteIdentity,
};

#[derive(Clone)]
pub(super) struct ScreenSpaceUiTextRouteContext {
    pub(super) tree_id: Arc<str>,
    pub(super) node_id: UiNodeId,
    pub(super) command_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextPlanOutcome {
    NotHandled,
    Planned,
    Rejected,
}

#[derive(Clone, Debug)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextBatch {
    pub(in crate::graphics::scene::scene_renderer::ui) route_identity:
        ScreenSpaceUiTextRouteIdentity,
    pub(in crate::graphics::scene::scene_renderer::ui) command_generation: u64,
    pub(in crate::graphics::scene::scene_renderer::ui) raster_scale: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) text: String,
    pub(in crate::graphics::scene::scene_renderer::ui) frame: UiFrame,
    pub(in crate::graphics::scene::scene_renderer::ui) clip_frame: Option<UiFrame>,
    pub(in crate::graphics::scene::scene_renderer::ui) source_range: Option<UiTextRange>,
    // This is planner provenance, not a heuristic: only a source-isomorphic resolved visual
    // line can safely host a native fallback span after an SDF atlas failure.
    pub(in crate::graphics::scene::scene_renderer::ui) is_source_isomorphic_layout_line: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) glyph_advances: Vec<f32>,
    pub(in crate::graphics::scene::scene_renderer::ui) shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
    // Runtime layout artifacts own these glyph identities. They must survive a font reload
    // without being replaced by a second, run-local shaping pass.
    pub(in crate::graphics::scene::scene_renderer::ui) preserve_shaped_glyphs: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) glyph_artifact_line:
        Option<ScreenSpaceUiGlyphArtifactLine>,
    pub(in crate::graphics::scene::scene_renderer::ui) layout_error: Option<TextLayoutError>,
    pub(in crate::graphics::scene::scene_renderer::ui) color: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::ui) background_color: Option<[f32; 4]>,
    pub(in crate::graphics::scene::scene_renderer::ui) font: Option<String>,
    pub(in crate::graphics::scene::scene_renderer::ui) font_family: Option<String>,
    pub(in crate::graphics::scene::scene_renderer::ui) language: Option<String>,
    pub(in crate::graphics::scene::scene_renderer::ui) font_weight: u16,
    pub(in crate::graphics::scene::scene_renderer::ui) font_size: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) line_height: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) text_align: UiTextAlign,
    pub(in crate::graphics::scene::scene_renderer::ui) text_direction: UiTextDirection,
    pub(in crate::graphics::scene::scene_renderer::ui) writing_mode: UiTextWritingMode,
    pub(in crate::graphics::scene::scene_renderer::ui) wrap: UiTextWrap,
    pub(in crate::graphics::scene::scene_renderer::ui) style: UiTextRunPaintStyle,
    pub(in crate::graphics::scene::scene_renderer::ui) distance_field_mode: SdfMode,
    pub(in crate::graphics::scene::scene_renderer::ui) text_effects: ScreenSpaceUiTextEffects,
    pub(in crate::graphics::scene::scene_renderer::ui) text_decorations:
        ScreenSpaceUiTextDecorations,
    pub(in crate::graphics::scene::scene_renderer::ui) text_decoration_baseline: Option<f32>,
    pub(in crate::graphics::scene::scene_renderer::ui) clip_transform:
        Option<super::text_projection::ScreenSpaceUiTextClipTransform>,
}

impl ScreenSpaceUiTextBatch {
    pub(super) fn preparation_inputs_match(&self, next: &Self) -> bool {
        self.route_identity == next.route_identity
            && self.command_generation == next.command_generation
            && self.raster_scale == next.raster_scale
            && self.text == next.text
            && self.frame == next.frame
            && self.clip_frame == next.clip_frame
            && self.source_range == next.source_range
            && self.is_source_isomorphic_layout_line == next.is_source_isomorphic_layout_line
            && self.glyph_advances == next.glyph_advances
            && self.shaped_glyphs == next.shaped_glyphs
            && self.preserve_shaped_glyphs == next.preserve_shaped_glyphs
            && self
                .glyph_artifact_line
                .as_ref()
                .map(ScreenSpaceUiGlyphArtifactLine::cache_identity)
                == next
                    .glyph_artifact_line
                    .as_ref()
                    .map(ScreenSpaceUiGlyphArtifactLine::cache_identity)
            && self.layout_error == next.layout_error
            && self.color == next.color
            && self.background_color == next.background_color
            && self.font == next.font
            && self.font_family == next.font_family
            && self.language == next.language
            && self.font_weight == next.font_weight
            && self.font_size == next.font_size
            && self.line_height == next.line_height
            && self.text_align == next.text_align
            && self.text_direction == next.text_direction
            && self.writing_mode == next.writing_mode
            && self.wrap == next.wrap
            && self.style == next.style
            && self.distance_field_mode == next.distance_field_mode
            && self.text_effects == next.text_effects
            && self.text_decorations == next.text_decorations
            && self.text_decoration_baseline == next.text_decoration_baseline
            && self.clip_transform == next.clip_transform
    }

    // A Text03 artifact retains the exact glyph sequence that native rasterization consumes.
    // Only visual fallback text without that artifact needs the SDF geometry owner.
    pub(in crate::graphics::scene::scene_renderer::ui) fn requires_sdf_layout_fidelity(
        &self,
    ) -> bool {
        self.glyph_artifact_line.is_none()
            && (!self.glyph_advances.is_empty()
                && self
                    .source_range
                    .is_some_and(|range| range.end.saturating_sub(range.start) != self.text.len()))
    }
}

/// 先消费解析布局或富文本的准入结果，再规划普通文字；不可恢复的布局或来源失配返回 Rejected。
/// 批次按布局保真、竖排、效果和显式模式分类；无距离场效果的 Auto 批次留到字体依赖加载后选路。
pub(super) fn push_text_batches(
    command: &UiRenderCommand,
    paint_elements: &[UiPaintElement],
    route_tree_id: &Arc<str>,
    route_node_id: UiNodeId,
    fallback_frame: UiFrame,
    color: [f32; 4],
    viewport: UiFrame,
    raster_scale: f32,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) -> TextPlanOutcome {
    if command.text.as_ref().is_some_and(|text| !text.is_empty())
        && !text_batch_frame_is_valid(command.frame)
    {
        plan.resolved_glyph_artifact_routes.record(
            resolved_layout::ResolvedGlyphArtifactRouteReceipt::Rejected(
                resolved_layout::ResolvedGlyphArtifactRejection::Incomplete,
            ),
        );
        return TextPlanOutcome::Rejected;
    }
    let route_context = ScreenSpaceUiTextRouteContext {
        tree_id: Arc::clone(route_tree_id),
        node_id: route_node_id,
        command_generation: command.cache_generation(),
    };
    if let Some(layout) = command.text_layout.as_ref() {
        if layout.lines.is_empty() {
            plan.resolved_glyph_artifact_routes.record(
                resolved_layout::ResolvedGlyphArtifactRouteReceipt::Rejected(
                    resolved_layout::ResolvedGlyphArtifactRejection::Incomplete,
                ),
            );
            return TextPlanOutcome::Rejected;
        }
        if !resolved_layout::resolved_text_layout_batch_geometry_is_valid(layout) {
            plan.resolved_glyph_artifact_routes.record(
                resolved_layout::ResolvedGlyphArtifactRouteReceipt::Rejected(
                    resolved_layout::ResolvedGlyphArtifactRejection::Incomplete,
                ),
            );
            return TextPlanOutcome::Rejected;
        }
        if matches!(
            command.style.rich_text_format,
            zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
        ) {
            let receipt = resolved_layout::push_resolved_text_layout_line_batches(
                command,
                &route_context,
                layout,
                color,
                viewport,
                raster_scale,
                backgrounds,
                plan,
            );
            let outcome = match receipt {
                resolved_layout::ResolvedGlyphArtifactRouteReceipt::Rejected(_) => {
                    TextPlanOutcome::Rejected
                }
                _ => TextPlanOutcome::Planned,
            };
            plan.resolved_glyph_artifact_routes.record(receipt);
            return outcome;
        }
    }

    if let Some(text_paint) = text_paint::command_text_paint(paint_elements) {
        match rich_text::plan_rich_text_runs(
            command,
            &route_context,
            text_paint,
            viewport,
            raster_scale,
            color,
            backgrounds,
            plan,
        ) {
            TextPlanOutcome::NotHandled => {}
            outcome => return outcome,
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
                &route_context,
                line.text.clone(),
                line.frame,
                Some(line.source_range),
                is_source_isomorphic_resolved_text_line(command, line),
                line.glyph_advances.clone(),
                None,
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
                raster_scale,
                backgrounds,
                plan,
            );
        }
        return TextPlanOutcome::Planned;
    }

    if let Some(text) = command.text.as_ref().filter(|text| !text.is_empty()) {
        if !text_batch_frame_is_valid(fallback_frame) {
            plan.resolved_glyph_artifact_routes.record(
                resolved_layout::ResolvedGlyphArtifactRouteReceipt::Rejected(
                    resolved_layout::ResolvedGlyphArtifactRejection::Incomplete,
                ),
            );
            return TextPlanOutcome::Rejected;
        }
        let font_size = command.style.font_size.max(1.0);
        push_text_batch(
            command,
            &route_context,
            text.clone(),
            fallback_frame,
            None,
            false,
            Vec::new(),
            None,
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
            raster_scale,
            backgrounds,
            plan,
        );
    }
    TextPlanOutcome::Planned
}

fn text_batch_frame_is_valid(frame: UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn push_text_batch(
    command: &UiRenderCommand,
    route_context: &ScreenSpaceUiTextRouteContext,
    text: String,
    frame: UiFrame,
    source_range: Option<UiTextRange>,
    is_source_isomorphic_layout_line: bool,
    glyph_advances: Vec<f32>,
    glyph_artifact_line: Option<ScreenSpaceUiGlyphArtifactLine>,
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
    raster_scale: f32,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    if text.is_empty() || !text_batch_frame_is_valid(frame) {
        return;
    }

    let language = text_language_cache_identity(command.style.language.as_deref());
    let has_glyph_artifact_line = glyph_artifact_line.is_some();
    let shaped_glyphs = Vec::new();

    let text_effects = command.style.text_effects.normalized();
    let distance_field_mode =
        resolved_text_distance_field_mode(command.style.text_render_mode, font_size, &text_effects);
    let text_decoration_baseline =
        resolved_text_decoration_baseline(command, source_range, writing_mode);
    let text_decorations = resolve_text_decorations(&text_decorations, color, command.opacity);
    let batch = ScreenSpaceUiTextBatch {
        route_identity: ScreenSpaceUiTextRouteIdentity::new(
            Arc::clone(&route_context.tree_id),
            route_context.node_id,
            source_range,
        ),
        command_generation: route_context.command_generation,
        raster_scale,
        text,
        frame,
        clip_frame: command.clip_frame,
        source_range,
        is_source_isomorphic_layout_line,
        glyph_advances,
        shaped_glyphs,
        preserve_shaped_glyphs: has_glyph_artifact_line,
        glyph_artifact_line,
        layout_error: None,
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
    if batch.requires_sdf_layout_fidelity()
        || matches!(batch.writing_mode, UiTextWritingMode::VerticalRl)
    {
        plan.sdf_texts.push(batch);
        return;
    }
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
