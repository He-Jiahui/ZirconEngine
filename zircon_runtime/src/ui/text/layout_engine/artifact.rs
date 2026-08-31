use crate::core::framework::text::TextLayoutError;
use crate::text::font::FontCollectionRevision;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{
    ResolvedTextGlyphArtifact, SharedTextLayoutSession, register_resolved_text_glyph_artifact,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::UiResolvedTextLayout;

#[derive(Clone, Copy)]
pub(super) struct LayoutFontGenerationFence {
    font_revision: FontCollectionRevision,
}

impl LayoutFontGenerationFence {
    pub(super) fn capture(provider: &SharedTextLayoutSession) -> Self {
        Self {
            font_revision: provider.font_collection_revision(),
        }
    }

    pub(super) fn ensure_current(
        self,
        provider: &SharedTextLayoutSession,
    ) -> TextLayoutOutcome<()> {
        self.ensure_revision(provider.font_collection_revision())
    }

    fn ensure_revision(self, current: FontCollectionRevision) -> TextLayoutOutcome<()> {
        if self.font_revision == current {
            TextShapingOutcome::Ready(())
        } else {
            TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged)
        }
    }

    #[cfg(test)]
    fn from_revision_for_test(font_revision: FontCollectionRevision) -> Self {
        Self { font_revision }
    }
}

/// Attaches an optional artifact only after its shaping result is known to be publishable.
///
/// A missing artifact is valid for visual-only resolved lines, but a deferred or failed shaping
/// result must reach the layout cache owner unchanged so it cannot publish partial geometry.
pub(super) fn attach_plain_text_glyph_artifact(
    layout: &mut UiResolvedTextLayout,
    artifact_outcome: TextLayoutOutcome<Option<ResolvedTextGlyphArtifact>>,
) -> TextLayoutOutcome<()> {
    match artifact_outcome {
        TextShapingOutcome::Ready(Some(artifact)) => {
            layout.rich_text_artifact =
                Some(register_resolved_text_glyph_artifact(Arc::new(artifact)));
            TextShapingOutcome::Ready(())
        }
        TextShapingOutcome::Ready(None) => TextShapingOutcome::Ready(()),
        TextShapingOutcome::Deferred(error) => TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => TextShapingOutcome::Failed(error),
    }
}

#[cfg(test)]
mod tests {
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
}
