use crate::core::math::Vec4;
use crate::text::{
    CompiledRichText, InlineBaseline, InlineObjectRef, StyledRun,
    resolve_compiled_rich_text_artifact,
};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedTextLine, UiTextAlign, UiTextDecorations, UiTextPaint,
    UiTextPaintRun, UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::super::image::ScreenSpaceUiImageBatch;
use super::background::ScreenSpaceUiBackgroundTracker;
use super::resolved_layout::{
    ResolvedGlyphArtifactRejection, ResolvedGlyphArtifactRouteReceipt, RichTextGlyphArtifactRoute,
    RichTextGlyphArtifactRouteBatch, rich_text_glyph_artifact_runs,
};
use super::text_batches::{TextPlanOutcome, push_text_batch};
use super::text_provenance::{SourceIsomorphicTextPaintLine, source_isomorphic_text_paint_line};
use super::{PlannedScreenSpaceUi, ScreenSpaceUiTextRouteContext, parse_color, push_rect};

#[cfg(feature = "profiling")]
const INLINE_FRAME_MATCH_TOLERANCE: f32 = 0.01;

#[derive(Default)]
struct RichInlineGeometryProfile {
    #[cfg(feature = "profiling")]
    inline_run_count: usize,
    #[cfg(feature = "profiling")]
    line_probe_count: usize,
    #[cfg(feature = "profiling")]
    line_run_probe_count: usize,
    #[cfg(feature = "profiling")]
    prefix_grapheme_count: usize,
    #[cfg(feature = "profiling")]
    prefix_advance_count: usize,
    #[cfg(feature = "profiling")]
    paint_frame_match_count: usize,
    #[cfg(feature = "profiling")]
    paint_frame_mismatch_count: usize,
}

impl RichInlineGeometryProfile {
    fn record_inline_run(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.inline_run_count = self.inline_run_count.saturating_add(1);
        }
    }

    fn record_line_probe(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.line_probe_count = self.line_probe_count.saturating_add(1);
        }
    }

    fn record_line_run_probe(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.line_run_probe_count = self.line_run_probe_count.saturating_add(1);
        }
    }

    fn record_prefix_work(&mut self, grapheme_count: usize, advance_count: usize) {
        #[cfg(feature = "profiling")]
        {
            self.prefix_grapheme_count = self.prefix_grapheme_count.saturating_add(grapheme_count);
            self.prefix_advance_count = self.prefix_advance_count.saturating_add(advance_count);
        }
        #[cfg(not(feature = "profiling"))]
        let _ = (grapheme_count, advance_count);
    }

    fn record_frame_comparison(
        &mut self,
        computed: Option<UiFrame>,
        paint_frame: UiFrame,
        writing_mode: UiTextWritingMode,
    ) {
        #[cfg(feature = "profiling")]
        {
            let matches = computed.is_some_and(|computed| {
                let difference = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
                    computed.y - paint_frame.y
                } else {
                    computed.x - paint_frame.x
                };
                difference.abs() <= INLINE_FRAME_MATCH_TOLERANCE
            });
            if matches {
                self.paint_frame_match_count = self.paint_frame_match_count.saturating_add(1);
            } else {
                self.paint_frame_mismatch_count = self.paint_frame_mismatch_count.saturating_add(1);
            }
        }
        #[cfg(not(feature = "profiling"))]
        let _ = (computed, paint_frame, writing_mode);
    }

    fn publish(&self) {
        #[cfg(feature = "profiling")]
        {
            crate::profile_counter!("runtime", "rich_inline_run_count", self.inline_run_count);
            crate::profile_counter!(
                "runtime",
                "rich_inline_line_probe_count",
                self.line_probe_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_inline_line_run_probe_count",
                self.line_run_probe_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_inline_prefix_grapheme_count",
                self.prefix_grapheme_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_inline_prefix_advance_count",
                self.prefix_advance_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_inline_paint_frame_match_count",
                self.paint_frame_match_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_inline_paint_frame_mismatch_count",
                self.paint_frame_mismatch_count
            );
        }
    }
}

pub(super) struct RichTextRunPresentation {
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub font_weight: u16,
    pub font_size: f32,
    pub line_height: f32,
    pub color: [f32; 4],
    pub text_decorations: UiTextDecorations,
}

struct RichTextRunAdmission<'paint, 'rich, 'layout> {
    run: &'paint UiTextPaintRun,
    artifact_route: RichTextGlyphArtifactRoute,
    source_rich_run: Option<&'rich StyledRun>,
    materialized_line: Option<SourceIsomorphicTextPaintLine<'layout>>,
    is_inline: bool,
}

struct RichTextRunAdmissionBatch<'paint, 'rich, 'layout> {
    runs: Vec<RichTextRunAdmission<'paint, 'rich, 'layout>>,
    command_rejection: Option<ResolvedGlyphArtifactRejection>,
}

#[derive(Default)]
struct RichTextRenderProfile {
    artifact_run_count: usize,
    visual_only_run_count: usize,
    source_isomorphic_fallback_run_count: usize,
    rejected_run_count: usize,
    missing_run_count: usize,
    stale_run_count: usize,
    incomplete_run_count: usize,
    fallback_shape_request_count: usize,
    fallback_shape_source_bytes: usize,
}

impl RichTextRenderProfile {
    fn record_rejection(&mut self, rejection: ResolvedGlyphArtifactRejection) {
        self.rejected_run_count = self.rejected_run_count.saturating_add(1);
        match rejection {
            ResolvedGlyphArtifactRejection::Missing => {
                self.missing_run_count = self.missing_run_count.saturating_add(1);
            }
            ResolvedGlyphArtifactRejection::Stale => {
                self.stale_run_count = self.stale_run_count.saturating_add(1);
            }
            ResolvedGlyphArtifactRejection::Incomplete => {
                self.incomplete_run_count = self.incomplete_run_count.saturating_add(1);
            }
        }
    }

    fn publish(&self) {
        crate::profile_counter!(
            "runtime",
            "rich_render_artifact_run_count",
            self.artifact_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_visual_only_run_count",
            self.visual_only_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_source_isomorphic_fallback_run_count",
            self.source_isomorphic_fallback_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_rejected_run_count",
            self.rejected_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_missing_run_count",
            self.missing_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_stale_run_count",
            self.stale_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_incomplete_run_count",
            self.incomplete_run_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_fallback_shape_request_count",
            self.fallback_shape_request_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_render_fallback_shape_source_bytes",
            self.fallback_shape_source_bytes
        );
    }
}

pub(super) fn lookup_command_rich_text(command: &UiRenderCommand) -> Option<Arc<CompiledRichText>> {
    resolve_compiled_rich_text_artifact(command.text_layout.as_ref()?.rich_text_artifact.as_ref()?)
}

pub(super) fn plan_rich_text_runs(
    command: &UiRenderCommand,
    route_context: &ScreenSpaceUiTextRouteContext,
    text_paint: &UiTextPaint,
    viewport: UiFrame,
    raster_scale: f32,
    fallback_color: [f32; 4],
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) -> TextPlanOutcome {
    if text_paint.runs.is_empty() && command.text_layout.is_none() {
        return TextPlanOutcome::NotHandled;
    }
    if !text_paint
        .runs
        .iter()
        .all(rich_text_paint_run_geometry_is_valid)
    {
        plan.resolved_glyph_artifact_routes
            .record(ResolvedGlyphArtifactRouteReceipt::Rejected(
                ResolvedGlyphArtifactRejection::Incomplete,
            ));
        return TextPlanOutcome::Rejected;
    }
    let parsed_rich = lookup_command_rich_text(command);
    let artifact_routes = match command.text_layout.as_ref() {
        None => (0..text_paint.runs.len())
            .map(|_| RichTextGlyphArtifactRoute::Rejected(ResolvedGlyphArtifactRejection::Missing))
            .collect::<Vec<_>>(),
        Some(layout) => match rich_text_glyph_artifact_runs(layout, &text_paint.runs) {
            RichTextGlyphArtifactRouteBatch::Complete(routes) => routes,
            RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch => {
                plan.resolved_glyph_artifact_routes.record(
                    ResolvedGlyphArtifactRouteReceipt::Rejected(
                        ResolvedGlyphArtifactRejection::Incomplete,
                    ),
                );
                return TextPlanOutcome::Rejected;
            }
        },
    };
    let admissions = preflight_rich_text_run_admissions(
        command,
        parsed_rich.as_deref(),
        &text_paint.runs,
        artifact_routes,
    );
    let mut render_profile = RichTextRenderProfile::default();
    for admission in &admissions.runs {
        plan.resolved_glyph_artifact_routes.record_rich_run(
            &admission.artifact_route,
            admission.materialized_line.is_some(),
        );
        if let RichTextGlyphArtifactRoute::Rejected(rejection) = &admission.artifact_route {
            if !admission.is_inline && admission.materialized_line.is_none() {
                render_profile.record_rejection(*rejection);
            }
        }
    }
    if let Some(rejection) = admissions.command_rejection {
        plan.resolved_glyph_artifact_routes
            .record(ResolvedGlyphArtifactRouteReceipt::Rejected(rejection));
        render_profile.publish();
        return TextPlanOutcome::Rejected;
    }
    let mut inline_geometry_profile = RichInlineGeometryProfile::default();

    for admission in admissions.runs {
        let run = admission.run;
        if admission.is_inline {
            let handled = plan_inline_run(
                command,
                run,
                admission.source_rich_run,
                viewport,
                plan,
                &mut inline_geometry_profile,
            );
            debug_assert!(handled, "preflighted inline run must remain inline");
            continue;
        }
        let materialized_line = admission.materialized_line;
        let source_isomorphic_fallback = materialized_line.is_some();
        let style_source_range = match &admission.artifact_route {
            RichTextGlyphArtifactRoute::Artifact(artifact_run) => artifact_run.style_source_range,
            _ => None,
        };
        let rich_run = style_source_range
            .and_then(|style_source_range| {
                parsed_rich
                    .as_ref()
                    .and_then(|parsed| run_for_range(parsed, style_source_range))
            })
            .or(admission.source_rich_run);
        let presentation = prepare_text_run(command, run, rich_run, viewport, fallback_color, plan);
        let glyph_artifact_line = match admission.artifact_route {
            RichTextGlyphArtifactRoute::Artifact(artifact_run) => {
                render_profile.artifact_run_count =
                    render_profile.artifact_run_count.saturating_add(1);
                Some(artifact_run.glyph_artifact_line)
            }
            RichTextGlyphArtifactRoute::VisualOnly => {
                render_profile.visual_only_run_count =
                    render_profile.visual_only_run_count.saturating_add(1);
                render_profile.fallback_shape_request_count = render_profile
                    .fallback_shape_request_count
                    .saturating_add(1);
                render_profile.fallback_shape_source_bytes = render_profile
                    .fallback_shape_source_bytes
                    .saturating_add(run.text.len());
                None
            }
            RichTextGlyphArtifactRoute::Rejected(rejection) => {
                debug_assert!(source_isomorphic_fallback);
                render_profile.source_isomorphic_fallback_run_count = render_profile
                    .source_isomorphic_fallback_run_count
                    .saturating_add(1);
                match rejection {
                    ResolvedGlyphArtifactRejection::Missing => {
                        render_profile.missing_run_count =
                            render_profile.missing_run_count.saturating_add(1);
                    }
                    ResolvedGlyphArtifactRejection::Stale => {
                        render_profile.stale_run_count =
                            render_profile.stale_run_count.saturating_add(1);
                    }
                    ResolvedGlyphArtifactRejection::Incomplete => {
                        render_profile.incomplete_run_count =
                            render_profile.incomplete_run_count.saturating_add(1);
                    }
                }
                render_profile.fallback_shape_request_count = render_profile
                    .fallback_shape_request_count
                    .saturating_add(1);
                render_profile.fallback_shape_source_bytes = render_profile
                    .fallback_shape_source_bytes
                    .saturating_add(run.text.len());
                None
            }
        };
        let (is_source_isomorphic_layout_line, text_align, text_direction, writing_mode, wrap) =
            materialized_line.map_or(
                (
                    false,
                    UiTextAlign::Left,
                    command.style.text_direction,
                    text_paint.writing_mode,
                    UiTextWrap::None,
                ),
                |materialized_line| {
                    (
                        true,
                        materialized_line.text_align,
                        materialized_line.line.direction,
                        materialized_line.writing_mode,
                        materialized_line.wrap,
                    )
                },
            );
        push_text_batch(
            command,
            route_context,
            run.text.clone(),
            run.frame,
            Some(run.source_range),
            is_source_isomorphic_layout_line,
            Vec::new(),
            glyph_artifact_line,
            presentation.font,
            presentation.font_family,
            presentation.font_weight,
            presentation.font_size,
            presentation.line_height,
            presentation.color,
            text_align,
            text_direction,
            writing_mode,
            wrap,
            run.style,
            presentation.text_decorations,
            viewport,
            raster_scale,
            backgrounds,
            plan,
        );
    }
    render_profile.publish();
    inline_geometry_profile.publish();
    TextPlanOutcome::Planned
}

fn preflight_rich_text_run_admissions<'paint, 'rich, 'layout>(
    command: &'layout UiRenderCommand,
    parsed_rich: Option<&'rich CompiledRichText>,
    paint_runs: &'paint [UiTextPaintRun],
    artifact_routes: Vec<RichTextGlyphArtifactRoute>,
) -> RichTextRunAdmissionBatch<'paint, 'rich, 'layout> {
    let mut command_rejection = None;
    let runs = paint_runs
        .iter()
        .zip(artifact_routes)
        .map(|(run, artifact_route)| {
            let source_rich_run =
                parsed_rich.and_then(|parsed| run_for_range(parsed, run.source_range));
            let is_inline = source_rich_run
                .and_then(|rich_run| rich_run.inline.as_ref())
                .is_some();
            let materialized_line = (!is_inline)
                .then(|| source_isomorphic_text_paint_line(command, run))
                .flatten();
            if let RichTextGlyphArtifactRoute::Rejected(rejection) = &artifact_route {
                if !is_inline && materialized_line.is_none() && command_rejection.is_none() {
                    command_rejection = Some(*rejection);
                }
            }
            RichTextRunAdmission {
                run,
                artifact_route,
                source_rich_run,
                materialized_line,
                is_inline,
            }
        })
        .collect();
    RichTextRunAdmissionBatch {
        runs,
        command_rejection,
    }
}

fn rich_text_paint_run_geometry_is_valid(run: &UiTextPaintRun) -> bool {
    let frame = run.frame;
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width >= 0.0
        && frame.height >= 0.0
        && run.font_size.is_finite()
        && run.font_size > 0.0
        && run.line_height.is_finite()
        && run.line_height > 0.0
}

pub(super) fn run_for_range(parsed: &CompiledRichText, range: UiTextRange) -> Option<&StyledRun> {
    if range.start == range.end {
        return None;
    }
    parsed.run_for_range(range.start, range.end)
}

pub(super) fn inline_frame(
    inline: &InlineObjectRef,
    run_frame: UiFrame,
    line: &UiResolvedTextLine,
    writing_mode: UiTextWritingMode,
) -> UiFrame {
    let (size, baseline) = match inline {
        InlineObjectRef::Image { size, baseline, .. }
        | InlineObjectRef::Icon { size, baseline, .. } => (*size, *baseline),
        InlineObjectRef::Widget { size, .. } => (*size, InlineBaseline::Baseline),
    };
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        let x = match baseline {
            InlineBaseline::Baseline | InlineBaseline::Center => {
                line.frame.x + (line.frame.width - size.x) * 0.5
            }
            InlineBaseline::Top => line.frame.x,
            InlineBaseline::Bottom => line.frame.right() - size.x,
        };
        UiFrame::new(x, run_frame.y, size.x, size.y)
    } else {
        let y = match baseline {
            InlineBaseline::Baseline => line.frame.y + line.baseline - size.y,
            InlineBaseline::Center => line.frame.y + (line.frame.height - size.y) * 0.5,
            InlineBaseline::Top => line.frame.y,
            InlineBaseline::Bottom => line.frame.bottom() - size.y,
        };
        UiFrame::new(run_frame.x, y, size.x, size.y)
    }
}

fn inline_layout_frame(
    line: &UiResolvedTextLine,
    inline: &InlineObjectRef,
    range: UiTextRange,
    writing_mode: UiTextWritingMode,
    profile: &mut RichInlineGeometryProfile,
) -> Option<UiFrame> {
    let visual_start = line.runs.iter().find_map(|run| {
        profile.record_line_run_probe();
        (run.source_range == range).then_some(run.visual_range.start)
    })?;
    let prefix = line.text.get(..visual_start)?;
    let grapheme_count = prefix.graphemes(true).count();
    let advance_count = grapheme_count.min(line.glyph_advances.len());
    profile.record_prefix_work(grapheme_count, advance_count);
    let main_offset = line.glyph_advances[..advance_count]
        .iter()
        .copied()
        .sum::<f32>();
    let run_frame = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        UiFrame::new(
            line.frame.x,
            line.frame.y + main_offset,
            line.frame.width,
            0.0,
        )
    } else {
        UiFrame::new(
            line.frame.x + main_offset,
            line.frame.y,
            0.0,
            line.frame.height,
        )
    };
    Some(inline_frame(inline, run_frame, line, writing_mode))
}

fn plan_inline_run(
    command: &UiRenderCommand,
    run: &UiTextPaintRun,
    rich_run: Option<&StyledRun>,
    viewport: UiFrame,
    plan: &mut PlannedScreenSpaceUi,
    profile: &mut RichInlineGeometryProfile,
) -> bool {
    let Some(inline) = rich_run.and_then(|rich_run| rich_run.inline.as_ref()) else {
        return false;
    };
    profile.record_inline_run();
    let Some(layout) = command.text_layout.as_ref() else {
        profile.record_frame_comparison(None, run.frame, command.style.text_writing_mode);
        return true;
    };
    let writing_mode = layout.writing_mode;
    let Some(line) = layout.lines.iter().find(|line| {
        profile.record_line_probe();
        line.source_range.start <= run.source_range.start
            && run.source_range.end <= line.source_range.end
    }) else {
        profile.record_frame_comparison(None, run.frame, writing_mode);
        return true;
    };
    let resolved_inline_frame =
        inline_layout_frame(line, inline, run.source_range, writing_mode, profile);
    profile.record_frame_comparison(resolved_inline_frame, run.frame, writing_mode);
    let inline_frame = resolved_inline_frame
        .unwrap_or_else(|| inline_frame(inline, run.frame, line, writing_mode));
    if viewport.intersection(inline_frame).is_none() {
        return true;
    }
    match inline {
        InlineObjectRef::Image { texture, .. } => plan.images.push(ScreenSpaceUiImageBatch {
            texture: *texture,
            frame: inline_frame,
            clip_frame: command.clip_frame,
            tint: [1.0, 1.0, 1.0, command.opacity.clamp(0.0, 1.0)],
        }),
        InlineObjectRef::Icon { asset, .. } => plan.images.push(ScreenSpaceUiImageBatch {
            texture: asset.resource_id(),
            frame: inline_frame,
            clip_frame: command.clip_frame,
            tint: [1.0, 1.0, 1.0, command.opacity.clamp(0.0, 1.0)],
        }),
        InlineObjectRef::Widget { .. } => {}
    }
    true
}

pub(super) fn prepare_text_run(
    command: &UiRenderCommand,
    run: &UiTextPaintRun,
    rich_run: Option<&StyledRun>,
    viewport: UiFrame,
    fallback_color: [f32; 4],
    plan: &mut PlannedScreenSpaceUi,
) -> RichTextRunPresentation {
    let font_size = rich_run
        .and_then(|rich_run| rich_run.style.font_size)
        .filter(|size| size.is_finite() && *size > 0.0)
        .unwrap_or(run.font_size);
    let color = rich_run
        .and_then(|rich_run| rich_run.style.color)
        .map(|color| rgba(color, command.opacity))
        .or_else(|| parse_color(run.color.as_deref(), fallback_color, command.opacity))
        .unwrap_or(fallback_color);
    if let Some(background) = rich_run
        .and_then(|rich_run| rich_run.style.bg_color)
        .map(|color| rgba(color, command.opacity))
    {
        if let Some(frame) = viewport.intersection(run.frame) {
            push_rect(&mut plan.vertices, frame, background, viewport);
        }
    }
    RichTextRunPresentation {
        font: run.font.clone().or_else(|| command.style.font.clone()),
        font_family: rich_run
            .and_then(|rich_run| rich_run.style.family.as_ref())
            .filter(|family| !family.is_empty())
            .map(|family| family.as_str().to_string())
            .or_else(|| run.font_family.clone())
            .or_else(|| command.style.font_family.clone()),
        font_weight: rich_run
            .and_then(|rich_run| rich_run.style.weight)
            .unwrap_or(run.font_weight),
        font_size,
        line_height: if run.font_size > 0.0 {
            run.line_height * (font_size / run.font_size)
        } else {
            run.line_height
        },
        color,
        text_decorations: decorations_for_rich_run(command, rich_run),
    }
}

pub(super) fn decorations_for_rich_run(
    command: &UiRenderCommand,
    rich_run: Option<&StyledRun>,
) -> UiTextDecorations {
    let mut decorations = command.style.text_decorations.clone();
    if let Some(rich_run) = rich_run {
        if let Some(underline) = rich_run.style.underline {
            decorations.underline = underline;
        }
        if let Some(strike) = rich_run.style.strike {
            decorations.strikethrough = strike;
        }
        if rich_run.link.is_some() {
            decorations.underline = true;
        }
    }
    decorations
}

pub(super) fn rgba(color: Vec4, opacity: f32) -> [f32; 4] {
    [
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
        (color.w * opacity).clamp(0.0, 1.0),
    ]
}
