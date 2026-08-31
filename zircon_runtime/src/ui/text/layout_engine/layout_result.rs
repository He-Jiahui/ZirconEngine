use std::sync::Arc;

use crate::text::layout::{CanonicalPhysicalLineFragment, LogicalVirtualLineSequence};
use zircon_runtime_interface::ui::surface::UiResolvedTextLayout;

/// Request-local layout state retained only until renderer artifact projection completes.
/// It must not escape into the serializable UI layout DTO or layout cache.
pub(super) struct LayoutWithoutArtifact {
    pub(super) layout: UiResolvedTextLayout,
    pub(super) retained_line_fragments: Option<Vec<Option<Arc<CanonicalPhysicalLineFragment>>>>,
    pub(super) retained_virtual_line_sequences: Option<Vec<Option<LogicalVirtualLineSequence>>>,
}

impl LayoutWithoutArtifact {
    pub(super) fn without_retained_fragments(layout: UiResolvedTextLayout) -> Self {
        Self {
            layout,
            retained_line_fragments: None,
            retained_virtual_line_sequences: None,
        }
    }

    pub(super) fn with_virtual_line_sequences(
        layout: UiResolvedTextLayout,
        sequences: Vec<Option<LogicalVirtualLineSequence>>,
    ) -> Self {
        let retained_virtual_line_sequences =
            sequences.iter().any(Option::is_some).then_some(sequences);
        Self {
            layout,
            retained_line_fragments: None,
            retained_virtual_line_sequences,
        }
    }
}
