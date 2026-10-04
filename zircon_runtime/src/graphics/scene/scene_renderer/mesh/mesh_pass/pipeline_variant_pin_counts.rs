use std::collections::{hash_map::Entry, HashMap};

use super::MeshPipelineVariantId;

/// Counts cross-frame command-cache entries that retain each pipeline variant.
///
/// Updates happen on the cache's existing insert/retain paths, so querying a
/// retirement candidate is O(1) and does not require another entry scan.
#[derive(Default)]
pub(super) struct PipelineVariantPinCounts {
    entry_counts: HashMap<MeshPipelineVariantId, usize>,
}

impl PipelineVariantPinCounts {
    pub(super) fn pin(&mut self, variant_id: MeshPipelineVariantId) {
        let count = self.entry_counts.entry(variant_id).or_default();
        *count = count
            .checked_add(1)
            .expect("pipeline variant pin count overflowed");
    }

    pub(super) fn unpin(&mut self, variant_id: MeshPipelineVariantId) {
        let Entry::Occupied(mut entry) = self.entry_counts.entry(variant_id) else {
            panic!("pipeline variant pin count must exist before unpin");
        };
        let count = entry.get_mut();
        *count = count
            .checked_sub(1)
            .expect("pipeline variant pin count must be positive before unpin");
        if *count == 0 {
            entry.remove();
        }
    }

    pub(super) fn replace(
        &mut self,
        previous: MeshPipelineVariantId,
        replacement: MeshPipelineVariantId,
    ) {
        if previous == replacement {
            return;
        }
        self.unpin(previous);
        self.pin(replacement);
    }

    pub(super) fn is_pinned(&self, variant_id: MeshPipelineVariantId) -> bool {
        self.entry_counts.contains_key(&variant_id)
    }

    pub(super) fn pinned_variant_count(&self) -> usize {
        self.entry_counts.len()
    }

    pub(super) fn clear(&mut self) {
        self.entry_counts.clear();
    }
}

#[cfg(test)]
#[path = "tests/pipeline_variant_pin_counts.rs"]
mod tests;
