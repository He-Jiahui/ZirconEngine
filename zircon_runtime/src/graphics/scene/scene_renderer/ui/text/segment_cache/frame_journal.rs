use std::sync::Arc;

use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiFrameChangeJournal;
use crate::text::font::FontCollectionRevision;

use super::super::ScreenSpaceUiTextFrameProductGeneration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) enum ScreenSpaceUiTextFrameFullRebuildReason {
    InitialPublication,
    SourceFullRebuild,
    SourceBaseGenerationMismatch,
    SourceTopologyMismatch,
    SourceChangeSetMismatch,
    RetainedStateUnavailable,
    ViewportChanged,
    FontRevisionChanged,
    FontAssetReloaded,
    ExplicitRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextFrameChangeJournal {
    base_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    current_generation: ScreenSpaceUiTextFrameProductGeneration,
    changed_segment_indices: Arc<[usize]>,
    appended_segment_count: usize,
    truncated_segment_count: usize,
    full_rebuild_reason: Option<ScreenSpaceUiTextFrameFullRebuildReason>,
}

pub(super) struct ScreenSpaceUiTextFrameJournalInputs<'a> {
    pub(super) source: &'a ScreenSpaceUiFrameChangeJournal,
    pub(super) previous_product_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    pub(super) previous_source_generation: Option<u64>,
    pub(super) previous_segment_count: usize,
    pub(super) previous_viewport_size: UVec2,
    pub(super) previous_font_revision: Option<FontCollectionRevision>,
    pub(super) retained_state_consistent: bool,
    pub(super) current_generation: ScreenSpaceUiTextFrameProductGeneration,
    pub(super) current_segment_count: usize,
    pub(super) current_viewport_size: UVec2,
    pub(super) current_font_revision: FontCollectionRevision,
    pub(super) changed_text_segment_indices: &'a [usize],
    pub(super) pending_full_rebuild_reason: Option<ScreenSpaceUiTextFrameFullRebuildReason>,
}

impl ScreenSpaceUiTextFrameChangeJournal {
    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::ui) fn for_test_initial(
        current_generation: ScreenSpaceUiTextFrameProductGeneration,
    ) -> Self {
        Self::full(
            current_generation,
            ScreenSpaceUiTextFrameFullRebuildReason::InitialPublication,
        )
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::ui) fn for_test_local(
        base_generation: ScreenSpaceUiTextFrameProductGeneration,
        current_generation: ScreenSpaceUiTextFrameProductGeneration,
        changed_segment_indices: &[usize],
        appended_segment_count: usize,
        truncated_segment_count: usize,
    ) -> Self {
        Self {
            base_generation: Some(base_generation),
            current_generation,
            changed_segment_indices: Arc::from(changed_segment_indices),
            appended_segment_count,
            truncated_segment_count,
            full_rebuild_reason: None,
        }
    }

    pub(super) fn publish(inputs: ScreenSpaceUiTextFrameJournalInputs<'_>) -> Self {
        let Some(previous_product_generation) = inputs.previous_product_generation else {
            return Self::full(
                inputs.current_generation,
                inputs
                    .pending_full_rebuild_reason
                    .unwrap_or(ScreenSpaceUiTextFrameFullRebuildReason::InitialPublication),
            );
        };
        if let Some(reason) = inputs.pending_full_rebuild_reason {
            return Self::full(inputs.current_generation, reason);
        }
        if inputs.previous_viewport_size != inputs.current_viewport_size {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::ViewportChanged,
            );
        }
        if inputs.previous_font_revision != Some(inputs.current_font_revision) {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::FontRevisionChanged,
            );
        }
        if !inputs.retained_state_consistent {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::RetainedStateUnavailable,
            );
        }
        if inputs.source.is_full_rebuild() {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::SourceFullRebuild,
            );
        }
        if inputs.source.base_generation() != inputs.previous_source_generation {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::SourceBaseGenerationMismatch,
            );
        }

        let appended_segment_count = inputs.source.appended_segment_count();
        let truncated_segment_count = inputs.source.truncated_segment_count();
        let topology_valid = truncated_segment_count <= inputs.previous_segment_count
            && inputs
                .previous_segment_count
                .saturating_sub(truncated_segment_count)
                .saturating_add(appended_segment_count)
                == inputs.current_segment_count;
        let retained_segment_count = inputs
            .previous_segment_count
            .saturating_sub(truncated_segment_count);
        let source_indices = inputs.source.changed_segment_indices();
        let indices_valid = indices_are_strictly_increasing(source_indices)
            && indices_are_strictly_increasing(inputs.changed_text_segment_indices)
            && source_indices
                .iter()
                .chain(inputs.changed_text_segment_indices)
                .all(|&index| index < retained_segment_count);
        if !topology_valid || !indices_valid {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::SourceTopologyMismatch,
            );
        }
        if !is_ordered_subset(inputs.changed_text_segment_indices, source_indices) {
            return Self::full(
                inputs.current_generation,
                ScreenSpaceUiTextFrameFullRebuildReason::SourceChangeSetMismatch,
            );
        }

        Self {
            base_generation: Some(previous_product_generation),
            current_generation: inputs.current_generation,
            changed_segment_indices: Arc::from(inputs.changed_text_segment_indices),
            appended_segment_count,
            truncated_segment_count,
            full_rebuild_reason: None,
        }
    }

    fn full(
        current_generation: ScreenSpaceUiTextFrameProductGeneration,
        reason: ScreenSpaceUiTextFrameFullRebuildReason,
    ) -> Self {
        Self {
            base_generation: None,
            current_generation,
            changed_segment_indices: Arc::from([]),
            appended_segment_count: 0,
            truncated_segment_count: 0,
            full_rebuild_reason: Some(reason),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn base_generation(
        &self,
    ) -> Option<ScreenSpaceUiTextFrameProductGeneration> {
        self.base_generation
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn current_generation(
        &self,
    ) -> ScreenSpaceUiTextFrameProductGeneration {
        self.current_generation
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn changed_segment_indices(
        &self,
    ) -> &[usize] {
        &self.changed_segment_indices
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn appended_segment_count(&self) -> usize {
        self.appended_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn truncated_segment_count(&self) -> usize {
        self.truncated_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn full_rebuild_reason(
        &self,
    ) -> Option<ScreenSpaceUiTextFrameFullRebuildReason> {
        self.full_rebuild_reason
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn is_full_rebuild(&self) -> bool {
        self.full_rebuild_reason.is_some()
    }
}

fn indices_are_strictly_increasing(indices: &[usize]) -> bool {
    indices.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_ordered_subset(subset: &[usize], superset: &[usize]) -> bool {
    let mut superset = superset.iter().copied();
    let mut current = superset.next();
    for &needle in subset {
        while current.is_some_and(|candidate| candidate < needle) {
            current = superset.next();
        }
        if current != Some(needle) {
            return false;
        }
        current = superset.next();
    }
    true
}

#[cfg(test)]
#[path = "tests/frame_journal.rs"]
mod tests;
