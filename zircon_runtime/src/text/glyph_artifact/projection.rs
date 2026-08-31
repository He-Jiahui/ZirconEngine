use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLine, UiTextDirection, UiTextWritingMode,
};

use crate::core::framework::text::{TextDirection, TextGlyph, TextLayoutError};
#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
use crate::text::font::FontHandleRegistrationBatchReport;
#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
use crate::text::font::register_font_handle_batch_with_report_for_collection;
use crate::text::font::{FontCollectionSnapshot, register_font_handle_batch_for_collection};
use crate::text::layout::LogicalVirtualLineSequence;
use crate::text::service::project_glyph;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{ShapedGlyph, ShapedGlyphRun, SharedTextLayoutSession, TextRange, VerticalMode};

use super::source_slice;
use super::visual_projection::visual_glyphs_for_visual_line;

pub(super) struct ArtifactShapedLine {
    pub(super) glyphs: Vec<TextGlyph>,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    pub(super) registration_report: Option<FontHandleRegistrationBatchReport>,
}

pub(super) fn shape_line_for_artifact(
    source_text: &str,
    source_text_origin: usize,
    style: &crate::text::TextStyle,
    writing_mode: UiTextWritingMode,
    line: &UiResolvedTextLine,
    provider: &mut SharedTextLayoutSession,
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<Option<ArtifactShapedLine>> {
    let Some(source) = source_slice(source_text, source_text_origin, line.source_range) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    let outcome = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        provider.shape_vertical_range(
            source,
            style,
            line.direction.into(),
            TextRange {
                start: line.source_range.start,
                end: line.source_range.end,
            },
            VerticalMode::Mixed,
        )
    } else {
        provider.shape_horizontal_range(
            source,
            style,
            line.direction.into(),
            TextRange {
                start: line.source_range.start,
                end: line.source_range.end,
            },
        )
    };
    let shaped = match outcome {
        TextShapingOutcome::Ready(shaped) => shaped,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let projected = match project_shaped_run_for_artifact(
        shaped.as_ref(),
        font_collection,
        collect_profile_metrics,
    ) {
        TextShapingOutcome::Ready(projected) => projected,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    TextShapingOutcome::Ready(Some(projected))
}

pub(super) fn shape_visual_line_for_artifact(
    source_text: &str,
    source_text_origin: usize,
    style: &crate::text::TextStyle,
    line: &UiResolvedTextLine,
    provider: &mut SharedTextLayoutSession,
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<Option<ArtifactShapedLine>> {
    if line.text.is_empty() || line.runs.is_empty() {
        return TextShapingOutcome::Ready(None);
    }
    let shaped = match provider.shape_horizontal_range(
        &line.text,
        style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: line.text.len(),
        },
    ) {
        TextShapingOutcome::Ready(shaped) => shaped,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let projected = match project_shaped_run_for_artifact(
        shaped.as_ref(),
        font_collection,
        collect_profile_metrics,
    ) {
        TextShapingOutcome::Ready(projected) => projected,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let Some(glyphs) =
        visual_glyphs_for_visual_line(source_text, source_text_origin, line, projected.glyphs)
    else {
        return TextShapingOutcome::Ready(None);
    };
    TextShapingOutcome::Ready(Some(ArtifactShapedLine {
        glyphs,
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        registration_report: projected.registration_report,
    }))
}

/// Projects the generated logical fragment through the UAX#9 order captured during layout.
///
/// The current-generation layout fragment is reused directly. A generation-invalid or absent
/// fragment keeps the established safe fallback: shape preserved logical input, never physical
/// resolved text, then project it through the retained logical order.
pub(super) fn shape_logical_virtual_line_for_artifact(
    sequence: &LogicalVirtualLineSequence,
    style: &crate::text::TextStyle,
    line: &UiResolvedTextLine,
    provider: &mut SharedTextLayoutSession,
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<Option<ArtifactShapedLine>> {
    if !sequence.artifact_projection_allowed() {
        return TextShapingOutcome::Ready(None);
    }
    let shaped = match sequence.fragment_for_revision(font_collection.revision()) {
        Some(fragment) => Arc::clone(fragment.shaped()),
        None => match provider.shape_horizontal_range(
            sequence.text(),
            style,
            sequence.base_direction(),
            TextRange {
                start: 0,
                end: sequence.text().len(),
            },
        ) {
            TextShapingOutcome::Ready(shaped) => shaped,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        },
    };
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let projected = match project_shaped_run_for_artifact(
        shaped.as_ref(),
        font_collection,
        collect_profile_metrics,
    ) {
        TextShapingOutcome::Ready(projected) => projected,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let Some(glyphs) = sequence.project_logical_glyphs(projected.glyphs, &line.glyph_advances)
    else {
        return TextShapingOutcome::Ready(None);
    };
    TextShapingOutcome::Ready(Some(ArtifactShapedLine {
        glyphs,
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        registration_report: projected.registration_report,
    }))
}

pub(super) fn line_uses_visual_artifact_projection(
    writing_mode: UiTextWritingMode,
    line: &UiResolvedTextLine,
) -> bool {
    // `line.text` has already been placed in physical order. Only an all-LTR line can use that
    // string as a fresh shaping input without losing RTL contextual shaping.
    matches!(writing_mode, UiTextWritingMode::HorizontalTb)
        && matches!(line.direction, UiTextDirection::LeftToRight)
        && line
            .runs
            .iter()
            .all(|run| matches!(run.direction, UiTextDirection::LeftToRight))
        && (line.ellipsized
            || line
                .runs
                .iter()
                .any(|run| !run.text.is_empty() && run.source_range.start == run.source_range.end))
}

pub(super) fn project_shaped_run_for_artifact(
    shaped: &ShapedGlyphRun,
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<ArtifactShapedLine> {
    project_shaped_run_with_vertical_origin_offsets_for_artifact(
        shaped,
        None,
        font_collection,
        collect_profile_metrics,
    )
}

/// Registers every font handle for one rich line as a single batch before projecting glyphs.
pub(super) fn project_shaped_runs_for_artifact(
    shaped_runs: &[Arc<ShapedGlyphRun>],
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<ArtifactShapedLine> {
    let glyphs = shaped_runs
        .iter()
        .flat_map(|run| run.lines.iter())
        .flat_map(|line| line.glyphs.iter())
        .collect::<Vec<_>>();
    project_glyphs_for_artifact(glyphs, None, font_collection, collect_profile_metrics)
}

/// Projects a cached shaped run without mutating its glyph origins.
///
/// A canonical line fragment may supply one vertical adjustment for every glyph. The adjustment
/// becomes part of the artifact-only raster origin because Native and SDF artifact renderers
/// consume `TextGlyph::offset[1]`, not `TextGlyph::position[1]`, for horizontal placement.
pub(super) fn project_shaped_run_with_vertical_origin_offsets_for_artifact(
    shaped: &ShapedGlyphRun,
    vertical_origin_offsets: Option<&[f32]>,
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<ArtifactShapedLine> {
    let glyphs = shaped
        .lines
        .iter()
        .flat_map(|line| line.glyphs.iter())
        .collect::<Vec<_>>();
    project_glyphs_for_artifact(
        glyphs,
        vertical_origin_offsets,
        font_collection,
        collect_profile_metrics,
    )
}

fn project_glyphs_for_artifact(
    source_glyphs: Vec<&ShapedGlyph>,
    vertical_origin_offsets: Option<&[f32]>,
    font_collection: &FontCollectionSnapshot,
    collect_profile_metrics: bool,
) -> TextLayoutOutcome<ArtifactShapedLine> {
    if font_collection.service().generation() != font_collection.generation() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let font_pairs = source_glyphs
        .iter()
        .map(|glyph| (glyph.font_id, glyph.font_instance_id))
        .collect::<Vec<_>>();
    if !vertical_origin_offsets_are_valid(vertical_origin_offsets, font_pairs.len()) {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    }
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let (handles, registration_report) = if collect_profile_metrics {
        let (handles, report) = register_font_handle_batch_with_report_for_collection(
            font_collection.service(),
            &font_pairs,
            font_collection.generation(),
        );
        (handles, Some(report))
    } else {
        (
            register_font_handle_batch_for_collection(
                font_collection.service(),
                &font_pairs,
                font_collection.generation(),
            ),
            None,
        )
    };
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let handles = {
        let _ = collect_profile_metrics;
        register_font_handle_batch_for_collection(
            font_collection.service(),
            &font_pairs,
            font_collection.generation(),
        )
    };
    if handles.len() != font_pairs.len()
        || font_collection.service().generation() != font_collection.generation()
    {
        return if font_collection.service().generation() != font_collection.generation() {
            TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged)
        } else {
            TextShapingOutcome::failed(TextLayoutError::LayoutFailed)
        };
    }
    let mut glyphs = source_glyphs
        .into_iter()
        .zip(handles)
        .map(|(glyph, handles)| project_glyph(glyph, handles))
        .collect::<Vec<_>>();
    if !apply_vertical_origin_offsets(&mut glyphs, vertical_origin_offsets) {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    }
    TextShapingOutcome::Ready(ArtifactShapedLine {
        glyphs,
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        registration_report,
    })
}

fn vertical_origin_offsets_are_valid(offsets: Option<&[f32]>, glyph_count: usize) -> bool {
    offsets.is_none_or(|offsets| {
        offsets.len() == glyph_count && offsets.iter().all(|offset| offset.is_finite())
    })
}

/// Applies only after validating all adjusted offsets so a bad sidecar cannot publish partial
/// vertical placement.
fn apply_vertical_origin_offsets(glyphs: &mut [TextGlyph], offsets: Option<&[f32]>) -> bool {
    let Some(offsets) = offsets else {
        return true;
    };
    if !vertical_origin_offsets_are_valid(Some(offsets), glyphs.len())
        || !glyphs
            .iter()
            .zip(offsets)
            .all(|(glyph, offset)| (glyph.offset[1] + offset).is_finite())
    {
        return false;
    }
    for (glyph, offset) in glyphs.iter_mut().zip(offsets) {
        glyph.offset[1] += offset;
    }
    true
}

/// Tracy streams counters continuously, while the CPU recorder should remain inert when idle.
pub(super) fn artifact_local_profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::text::{TextGlyphFlags, TextGlyphRotation};

    use super::{TextGlyph, apply_vertical_origin_offsets};

    fn glyph(offset_y: f32) -> TextGlyph {
        TextGlyph {
            glyph_id: 7,
            source_range: 0..1,
            visual_range: 0..1,
            advance: 4.0,
            position: [0.0, 0.0],
            offset: [0.25, offset_y],
            font_face: None,
            font_instance: None,
            rotation: TextGlyphRotation::None,
            bidi_level: 0,
            flags: TextGlyphFlags::default(),
            requires_rasterization: false,
        }
    }

    #[test]
    fn vertical_origin_offsets_adjust_only_artifact_glyph_offsets() {
        let mut glyphs = vec![glyph(-1.0), glyph(2.0)];

        assert!(apply_vertical_origin_offsets(
            &mut glyphs,
            Some(&[0.5, -1.25])
        ));
        assert_eq!(glyphs[0].offset, [0.25, -0.5]);
        assert_eq!(glyphs[1].offset, [0.25, 0.75]);
        assert_eq!(glyphs[0].position, [0.0, 0.0]);
        assert_eq!(glyphs[1].position, [0.0, 0.0]);
    }

    #[test]
    fn invalid_vertical_origin_sidecar_leaves_artifact_glyphs_unchanged() {
        let mut glyphs = vec![glyph(-1.0), glyph(2.0)];
        let original = glyphs.clone();

        assert!(!apply_vertical_origin_offsets(&mut glyphs, Some(&[0.5])));
        assert_eq!(glyphs, original);
        assert!(!apply_vertical_origin_offsets(
            &mut glyphs,
            Some(&[0.5, f32::NAN])
        ));
        assert_eq!(glyphs, original);
        let mut overflowing_glyphs = vec![glyph(f32::MAX), glyph(2.0)];
        let overflowing_original = overflowing_glyphs.clone();
        assert!(!apply_vertical_origin_offsets(
            &mut overflowing_glyphs,
            Some(&[f32::MAX, 0.0])
        ));
        assert_eq!(overflowing_glyphs, overflowing_original);
    }
}
