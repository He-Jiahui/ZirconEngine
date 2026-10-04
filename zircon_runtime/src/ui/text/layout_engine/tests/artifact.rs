use super::*;

#[test]
fn nonready_artifact_outcomes_do_not_become_publishable_layouts() {
    let mut deferred_layout = UiResolvedTextLayout::default();
    assert!(matches!(
        attach_plain_text_glyph_artifact(
            &mut deferred_layout,
            TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged),
        ),
        TextShapingOutcome::Deferred(failure)
            if failure.error() == &TextLayoutError::FontGenerationChanged
                && failure.receipt().is_some()
    ));
    assert!(deferred_layout.rich_text_artifact.is_none());

    let mut failed_layout = UiResolvedTextLayout::default();
    assert!(matches!(
        attach_plain_text_glyph_artifact(
            &mut failed_layout,
            TextShapingOutcome::failed(TextLayoutError::InvalidFontSize),
        ),
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::InvalidFontSize
    ));
    assert!(failed_layout.rich_text_artifact.is_none());
}

#[test]
fn ready_without_artifact_remains_a_publishable_dto_layout() {
    let mut layout = UiResolvedTextLayout::default();
    assert!(matches!(
        attach_plain_text_glyph_artifact(&mut layout, TextShapingOutcome::Ready(None)),
        TextShapingOutcome::Ready(())
    ));
    assert!(layout.rich_text_artifact.is_none());
}

#[test]
fn retired_layout_font_generation_is_not_publishable() {
    let current = crate::text::font::shared_font_collection_service().revision();
    let retired = FontCollectionRevision::new(
        current.collection_id(),
        current.generation().saturating_sub(1),
    );
    let fence = LayoutFontGenerationFence::from_revision_for_test(retired);

    assert!(matches!(
        fence.ensure_revision(current),
        TextShapingOutcome::Deferred(failure)
            if failure.error() == &TextLayoutError::FontGenerationChanged
                && failure.receipt().is_some()
    ));
}
