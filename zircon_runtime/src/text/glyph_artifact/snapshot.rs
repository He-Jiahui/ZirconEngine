use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine,
};

use crate::core::framework::text::TextGlyph;

use super::super::font::FontCollectionRevision;
use super::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine,
    resolved_text_line_requires_visual_fallback, source_text_origin,
};

pub(crate) fn resolved_text_glyph_artifact_matches_layout_snapshot(
    artifact: &ResolvedTextGlyphArtifact,
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    font_revision: FontCollectionRevision,
) -> bool {
    artifact.source_text.as_ref() == source_text
        && source_text_origin(source_text, layout.source_range)
            .is_some_and(|origin| artifact.source_text_origin == origin)
        && artifact.font_lease.revision() == font_revision
        && artifact.font_generation == font_revision.generation()
        && artifact.style == *style
        && artifact.writing_mode == layout.writing_mode
        && artifact.lines.len() == layout.lines.len()
        && artifact
            .lines
            .iter()
            .zip(&layout.lines)
            .all(|(artifact_line, layout_line)| match artifact_line {
                Some(artifact_line) => artifact_line.layout_line == *layout_line,
                None => resolved_text_line_requires_visual_fallback(layout_line),
            })
}

pub(crate) fn resolved_text_glyph_artifact_line_matches_layout(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
) -> bool {
    matching_artifact_line_entry(artifact, line_index, layout_line).is_some()
}

pub(super) fn matching_artifact_line<'a>(
    artifact: &'a ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
) -> Option<&'a [TextGlyph]> {
    (artifact.font_generation == artifact.font_lease.generation())
        .then(|| matching_artifact_line_entry(artifact, line_index, layout_line))
        .flatten()
        .map(|line| line.glyphs.as_slice())
}

fn matching_artifact_line_entry<'a>(
    artifact: &'a ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
) -> Option<&'a ResolvedTextGlyphArtifactLine> {
    artifact
        .lines
        .get(line_index)?
        .as_ref()
        .filter(|artifact_line| artifact_line.layout_line == *layout_line)
}
