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
    let handle = register_resolved_rich_text_artifact(Arc::clone(&compiled), Arc::clone(&glyphs));

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
