use std::sync::Arc;

use crate::text::layout::LogicalVirtualLineSequence;

use super::ResolvedTextGlyphArtifact;

#[derive(Clone, Debug)]
pub(super) struct ResolvedTextGlyphArtifactIdentity(Arc<ResolvedTextGlyphArtifact>);

impl ResolvedTextGlyphArtifactIdentity {
    pub(super) fn new(artifact: Arc<ResolvedTextGlyphArtifact>) -> Self {
        Self(artifact)
    }
}

impl PartialEq for ResolvedTextGlyphArtifactIdentity {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let left = self.0.as_ref();
        let right = other.0.as_ref();
        left.source_text == right.source_text
            && left.source_text_origin == right.source_text_origin
            && left.font_generation == right.font_generation
            && left.font_lease.revision() == right.font_lease.revision()
            && left.style == right.style
            && left.writing_mode == right.writing_mode
            && left.lines == right.lines
            && logical_virtual_line_sequences_have_same_identity(
                left.logical_virtual_line_sequences.as_deref(),
                right.logical_virtual_line_sequences.as_deref(),
            )
    }
}

fn logical_virtual_line_sequences_have_same_identity(
    left: Option<&[Option<LogicalVirtualLineSequence>]>,
    right: Option<&[Option<LogicalVirtualLineSequence>]>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) if left.len() == right.len() => {
            left.iter()
                .zip(right)
                .all(|(left, right)| match (left, right) {
                    (None, None) => true,
                    (Some(left), Some(right)) => left.has_same_artifact_identity(right),
                    (None, Some(_)) | (Some(_), None) => false,
                })
        }
        (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => false,
    }
}
