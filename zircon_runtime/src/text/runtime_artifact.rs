use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle,
    UiTextRange,
};

use super::layout::LogicalVirtualLineSequence;

#[cfg(test)]
use super::ResolvedTextGlyphArtifactFontLease;
use super::{
    CompiledRichText, ResolvedTextGlyphArtifact, register_compiled_rich_text_artifact,
    register_resolved_text_glyph_artifact,
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
mod tests {
    use super::*;
    use crate::text::{RichTextFormat, RichTextParser};
    use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextWritingMode};

    fn compile_rich(
        parser: &RichTextParser,
        markup: &str,
        format: RichTextFormat,
    ) -> Arc<CompiledRichText> {
        parser
            .compile(markup, format)
            .expect("test rich source fits parser budgets")
    }

    fn glyph_artifact(source_text: &'static str) -> Arc<ResolvedTextGlyphArtifact> {
        Arc::new(ResolvedTextGlyphArtifact {
            source_text: Arc::from(source_text),
            source_text_origin: 0,
            font_generation: 7,
            font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
            style: UiResolvedStyle::default(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            lines: Vec::new(),
            logical_virtual_line_sequences: None,
        })
    }

    fn glyph_run(glyph_range: std::ops::Range<usize>) -> Arc<[ResolvedRichTextGlyphRun]> {
        glyph_run_with_style(glyph_range, None)
    }

    fn glyph_run_with_style(
        glyph_range: std::ops::Range<usize>,
        style_source_range: Option<UiTextRange>,
    ) -> Arc<[ResolvedRichTextGlyphRun]> {
        Arc::from([ResolvedRichTextGlyphRun {
            line_index: 0,
            source_range: UiTextRange { start: 0, end: 4 },
            visual_range: UiTextRange { start: 0, end: 4 },
            style_source_range,
            replaced_source_range: None,
            glyph_range,
        }])
    }

    #[test]
    fn composite_rich_artifact_resolves_interaction_and_glyph_products() {
        let parser = RichTextParser::default();
        let compiled = compile_rich(&parser, "[url=docs]text[/url]", RichTextFormat::BbCodeV1);
        assert!(compiled.parsed().runs.iter().any(|run| {
            run.link
                .as_ref()
                .is_some_and(|link| link.target.matches_display("res://docs"))
        }));
        let glyphs = glyph_artifact("text");
        let handle =
            register_resolved_rich_text_artifact(Arc::clone(&compiled), Arc::clone(&glyphs));

        assert!(Arc::ptr_eq(
            &resolve_compiled_rich_text_from_composite(&handle).expect("compiled rich text"),
            &compiled,
        ));
        assert!(Arc::ptr_eq(
            &resolve_text_glyphs_from_composite(&handle).expect("glyph artifact"),
            &glyphs,
        ));
    }

    #[test]
    fn composite_rich_artifact_identity_tracks_both_products() {
        let parser = RichTextParser::default();
        let first = register_resolved_rich_text_artifact(
            compile_rich(&parser, "[url=docs]text[/url]", RichTextFormat::BbCodeV1),
            glyph_artifact("text"),
        );
        let same = register_resolved_rich_text_artifact(
            compile_rich(&parser, "[url=docs]text[/url]", RichTextFormat::BbCodeV1),
            glyph_artifact("text"),
        );
        let different_compiled = register_resolved_rich_text_artifact(
            compile_rich(&parser, "[url=other]text[/url]", RichTextFormat::BbCodeV1),
            glyph_artifact("text"),
        );
        let different_glyphs = register_resolved_rich_text_artifact(
            compile_rich(&parser, "[url=docs]text[/url]", RichTextFormat::BbCodeV1),
            glyph_artifact("different"),
        );

        assert_eq!(first, same);
        assert_ne!(first, different_compiled);
        assert_ne!(first, different_glyphs);
    }

    #[test]
    fn composite_rich_artifact_resolves_run_slice_and_tracks_its_identity() {
        let parser = RichTextParser::default();
        let compiled = compile_rich(&parser, "[url=docs]text[/url]", RichTextFormat::BbCodeV1);
        let glyphs = glyph_artifact("text");
        let first = register_resolved_rich_text_artifact_with_runs(
            Arc::clone(&compiled),
            Arc::clone(&glyphs),
            glyph_run_with_style(0..1, Some(UiTextRange { start: 0, end: 4 })),
        );
        let different_run =
            register_resolved_rich_text_artifact_with_runs(compiled, glyphs, glyph_run(1..2));

        let resolved = resolve_rich_text_glyph_run_artifact(
            &first,
            0,
            UiTextRange { start: 0, end: 4 },
            UiTextRange { start: 0, end: 4 },
        )
        .expect("mapped rich glyph run");
        assert_eq!(resolved.line_index, 0);
        assert_eq!(resolved.glyph_range, 0..1);
        assert_eq!(
            resolved.style_source_range,
            Some(UiTextRange { start: 0, end: 4 })
        );
        assert!(
            resolve_rich_text_glyph_run_artifact_at(
                &first,
                1,
                0,
                UiTextRange { start: 0, end: 4 },
                UiTextRange { start: 0, end: 4 },
            )
            .is_none(),
            "an out-of-directory index must fail closed"
        );
        assert!(
            resolve_rich_text_glyph_run_artifact_at(
                &first,
                0,
                0,
                UiTextRange { start: 1, end: 4 },
                UiTextRange { start: 0, end: 4 },
            )
            .is_none(),
            "directory lookup must validate the exact run identity"
        );
        assert_ne!(first, different_run);
    }
}
