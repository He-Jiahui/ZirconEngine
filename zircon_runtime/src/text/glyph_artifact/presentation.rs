use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextWritingMode,
};

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::text::font::FontCollectionSnapshot;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{SharedTextLayoutSession, TextRange, text_style};

use super::projection::{artifact_local_profile_metrics_enabled, project_shaped_run_for_artifact};
use super::visual_projection::presentation_glyphs_for_line;
use super::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactFontLease, ResolvedTextGlyphArtifactLine,
};

/// Shapes an already visual-order secure mask and projects every resulting glyph back to the
/// source ranges retained by the layout's one-grapheme presentation runs. `display_text` is the
/// only text retained in the artifact; the original input never crosses this boundary.
pub(crate) fn build_resolved_text_presentation_glyph_artifact(
    display_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<ResolvedTextGlyphArtifact>> {
    if !matches!(layout.writing_mode, UiTextWritingMode::HorizontalTb)
        || layout.lines.iter().any(|line| line.ellipsized)
    {
        return TextShapingOutcome::Ready(None);
    }
    let font_collection = provider.font_collection_snapshot();
    let font_revision = font_collection.revision();
    let font_generation = font_collection.generation();
    let shaped_style = text_style(&UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    });
    let artifact_style = UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    };
    let mut lines = Vec::with_capacity(layout.lines.len());
    for line in &layout.lines {
        let projected =
            match shape_presentation_line(line, &shaped_style, provider, &font_collection) {
                TextShapingOutcome::Ready(Some(projected)) => projected,
                TextShapingOutcome::Ready(None) => return TextShapingOutcome::Ready(None),
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
        lines.push(Some(ResolvedTextGlyphArtifactLine {
            glyphs: projected,
            layout_line: line.clone(),
        }));
    }
    if lines.is_empty()
        || !lines.iter().any(Option::is_some)
        || provider.font_collection_revision() != font_revision
    {
        return if provider.font_collection_revision() != font_revision {
            TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged)
        } else {
            TextShapingOutcome::Ready(None)
        };
    }
    TextShapingOutcome::Ready(Some(ResolvedTextGlyphArtifact {
        source_text: Arc::from(display_text),
        source_text_origin: 0,
        font_generation,
        font_lease: ResolvedTextGlyphArtifactFontLease::capture(font_collection),
        style: artifact_style,
        writing_mode: layout.writing_mode,
        lines,
        logical_virtual_line_sequences: None,
    }))
}

fn shape_presentation_line(
    line: &UiResolvedTextLine,
    style: &crate::text::TextStyle,
    provider: &mut SharedTextLayoutSession,
    font_collection: &FontCollectionSnapshot,
) -> TextLayoutOutcome<Option<Vec<crate::core::framework::text::TextGlyph>>> {
    if line.text.is_empty() || line.glyph_advances.len() != line.runs.len() {
        return TextShapingOutcome::Ready(None);
    }
    let shaped = match provider.shape_horizontal_range(
        &line.text,
        style,
        // `line.text` has already been ordered physically by the presentation owner.
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
    if provider.font_collection_revision() != font_collection.revision() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    let projected = match project_shaped_run_for_artifact(
        shaped.as_ref(),
        font_collection,
        artifact_local_profile_metrics_enabled(),
    ) {
        TextShapingOutcome::Ready(projected) => projected,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if provider.font_collection_revision() != font_collection.revision() {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    TextShapingOutcome::Ready(presentation_glyphs_for_line(line, projected.glyphs))
}
