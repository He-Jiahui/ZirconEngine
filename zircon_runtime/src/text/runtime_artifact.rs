//! 组合富文本工件的注册、身份和按行字形目录解析。

use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle,
    UiTextRange,
};

use super::layout::LogicalVirtualLineSequence;

#[cfg(test)]
use super::ResolvedTextGlyphArtifactFontLease;
use super::{
    register_compiled_rich_text_artifact, register_resolved_text_glyph_artifact, CompiledRichText,
    ResolvedTextGlyphArtifact,
};

/// Process-local rich text product shared by input and rendering consumers.
///
/// The public UI DTO intentionally carries one opaque handle. Rich text therefore keeps its
/// compiled interaction metadata and immutable shaped-glyph sidecar in one owner allocation.
pub(crate) struct ResolvedRichTextArtifact {
    compiled: Arc<CompiledRichText>,
    glyphs: Arc<ResolvedTextGlyphArtifact>,
    layout_lines: Arc<[UiResolvedTextLine]>,
    glyph_runs: Arc<[ResolvedRichTextGlyphRun]>,
}

#[derive(Clone, PartialEq)]
struct ResolvedRichTextArtifactIdentity {
    compiled: UiRichTextArtifactHandle,
    glyphs: UiRichTextArtifactHandle,
    layout_lines: Arc<[UiResolvedTextLine]>,
    glyph_runs: Arc<[ResolvedRichTextGlyphRun]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedRichTextGlyphRun {
    pub(crate) line_index: usize,
    pub(crate) source_range: UiTextRange,
    pub(crate) visual_range: UiTextRange,
    pub(crate) style_source_range: Option<UiTextRange>,
    pub(crate) replaced_source_range: Option<UiTextRange>,
    pub(crate) glyph_range: std::ops::Range<usize>,
}

pub(crate) struct ResolvedRichTextGlyphRunArtifact {
    pub(crate) artifact: Arc<ResolvedTextGlyphArtifact>,
    pub(crate) line_index: usize,
    pub(crate) style_source_range: Option<UiTextRange>,
    pub(crate) glyph_range: std::ops::Range<usize>,
}

pub(crate) fn register_resolved_rich_text_artifact(
    compiled: Arc<CompiledRichText>,
    glyphs: Arc<ResolvedTextGlyphArtifact>,
) -> UiRichTextArtifactHandle {
    register_resolved_rich_text_artifact_with_layout_runs(
        compiled,
        glyphs,
        Arc::from([]),
        Arc::from([]),
    )
}

pub(crate) fn register_resolved_rich_text_artifact_with_runs(
    compiled: Arc<CompiledRichText>,
    glyphs: Arc<ResolvedTextGlyphArtifact>,
    glyph_runs: Arc<[ResolvedRichTextGlyphRun]>,
) -> UiRichTextArtifactHandle {
    register_resolved_rich_text_artifact_with_layout_runs(
        compiled,
        glyphs,
        Arc::from([]),
        glyph_runs,
    )
}

pub(crate) fn register_resolved_rich_text_artifact_with_layout_runs(
    compiled: Arc<CompiledRichText>,
    glyphs: Arc<ResolvedTextGlyphArtifact>,
    layout_lines: Arc<[UiResolvedTextLine]>,
    glyph_runs: Arc<[ResolvedRichTextGlyphRun]>,
) -> UiRichTextArtifactHandle {
    // 同一 opaque handle 保留语义工件、字形工件和行目录，交互与渲染消费者共享这组产物。
    let identity = ResolvedRichTextArtifactIdentity {
        compiled: register_compiled_rich_text_artifact(Arc::clone(&compiled)),
        glyphs: register_resolved_text_glyph_artifact(Arc::clone(&glyphs)),
        layout_lines: Arc::clone(&layout_lines),
        glyph_runs: Arc::clone(&glyph_runs),
    };
    UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
        Arc::new(ResolvedRichTextArtifact {
            compiled,
            glyphs,
            layout_lines,
            glyph_runs,
        }),
        identity,
    )
}

pub(crate) fn resolved_rich_text_artifact_matches_layout_snapshot(
    handle: &UiRichTextArtifactHandle,
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    font_revision: crate::text::font::FontCollectionRevision,
) -> bool {
    let Some(artifact) = handle.downcast_runtime_artifact::<ResolvedRichTextArtifact>() else {
        return false;
    };
    rich_text_artifact_matches_layout(artifact.as_ref(), source_text, style, layout)
        && artifact.glyphs.font_lease.revision() == font_revision
        && artifact.glyphs.font_generation == font_revision.generation()
}

pub(crate) fn resolve_rich_text_virtual_line_sequences_for_layout(
    handle: &UiRichTextArtifactHandle,
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
) -> Option<Vec<Option<LogicalVirtualLineSequence>>> {
    let artifact = handle.downcast_runtime_artifact::<ResolvedRichTextArtifact>()?;
    rich_text_artifact_matches_layout(artifact.as_ref(), source_text, style, layout)
        .then(|| artifact.glyphs.logical_virtual_line_sequences.clone())
        .flatten()
}

fn rich_text_artifact_matches_layout(
    artifact: &ResolvedRichTextArtifact,
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
) -> bool {
    let glyphs = artifact.glyphs.as_ref();
    glyphs.source_text.as_ref() == source_text
        && crate::text::glyph_artifact::source_text_origin(source_text, layout.source_range)
            .is_some_and(|origin| glyphs.source_text_origin == origin)
        && glyphs.style == *style
        && glyphs.writing_mode == layout.writing_mode
        && glyphs.lines.len() == layout.lines.len()
        && artifact.layout_lines.as_ref() == layout.lines.as_slice()
}

pub(super) fn resolve_compiled_rich_text_from_composite(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<CompiledRichText>> {
    handle
        .downcast_runtime_artifact::<ResolvedRichTextArtifact>()
        .map(|artifact| Arc::clone(&artifact.compiled))
}

pub(super) fn resolve_text_glyphs_from_composite(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<ResolvedTextGlyphArtifact>> {
    handle
        .downcast_runtime_artifact::<ResolvedRichTextArtifact>()
        .map(|artifact| Arc::clone(&artifact.glyphs))
}

pub(crate) fn resolve_rich_text_glyph_run_artifact(
    handle: &UiRichTextArtifactHandle,
    line_index: usize,
    source_range: UiTextRange,
    visual_range: UiTextRange,
) -> Option<ResolvedRichTextGlyphRunArtifact> {
    let artifact = handle.downcast_runtime_artifact::<ResolvedRichTextArtifact>()?;
    let directory_index = artifact.glyph_runs.iter().position(|run| {
        run.line_index == line_index
            && run.source_range == source_range
            && run.visual_range == visual_range
    })?;
    resolve_rich_text_glyph_run_artifact_at(
        handle,
        directory_index,
        line_index,
        source_range,
        visual_range,
    )
}

pub(crate) fn resolve_rich_text_glyph_run_artifact_at(
    handle: &UiRichTextArtifactHandle,
    directory_index: usize,
    line_index: usize,
    source_range: UiTextRange,
    visual_range: UiTextRange,
) -> Option<ResolvedRichTextGlyphRunArtifact> {
    // 以目录序号定位后仍核验行号、源范围和视觉范围，避免重复 run 的条目被误用。
    let artifact = handle.downcast_runtime_artifact::<ResolvedRichTextArtifact>()?;
    let run = artifact.glyph_runs.get(directory_index)?;
    if run.line_index != line_index
        || run.source_range != source_range
        || run.visual_range != visual_range
    {
        return None;
    }
    if run.replaced_source_range.is_some_and(|range| {
        artifact.layout_lines.get(line_index).is_none_or(|line| {
            range.start >= range.end
                || range.start < line.source_range.start
                || range.end > line.source_range.end
        })
    }) {
        return None;
    }
    Some(ResolvedRichTextGlyphRunArtifact {
        artifact: Arc::clone(&artifact.glyphs),
        line_index,
        style_source_range: run.style_source_range,
        glyph_range: run.glyph_range.clone(),
    })
}

#[cfg(test)]
#[path = "tests/runtime_artifact.rs"]
mod tests;
