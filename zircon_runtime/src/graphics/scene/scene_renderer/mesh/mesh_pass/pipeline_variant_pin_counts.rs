use std::collections::{HashMap, hash_map::Entry};

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
mod tests {
    use super::PipelineVariantPinCounts;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId;

    fn variant(value: u32) -> MeshPipelineVariantId {
        MeshPipelineVariantId::new(value)
    }

    #[test]
    fn repeated_cache_entries_keep_one_variant_pinned_until_the_last_unpin() {
        let mut pins = PipelineVariantPinCounts::default();
        let variant = variant(7);

        pins.pin(variant);
        pins.pin(variant);
        assert!(pins.is_pinned(variant));
        assert_eq!(pins.pinned_variant_count(), 1);

        pins.unpin(variant);
        assert!(pins.is_pinned(variant));
        pins.unpin(variant);
        assert!(!pins.is_pinned(variant));
        assert_eq!(pins.pinned_variant_count(), 0);
    }

    #[test]
    fn replacing_with_the_same_variant_does_not_change_counts() {
        let mut pins = PipelineVariantPinCounts::default();
        let variant = variant(11);

        pins.pin(variant);
        pins.replace(variant, variant);

        assert!(pins.is_pinned(variant));
        assert_eq!(pins.pinned_variant_count(), 1);
    }

    #[test]
    fn replacing_with_a_different_variant_moves_the_pin() {
        let mut pins = PipelineVariantPinCounts::default();
        let previous = variant(13);
        let replacement = variant(17);

        pins.pin(previous);
        pins.replace(previous, replacement);

        assert!(!pins.is_pinned(previous));
        assert!(pins.is_pinned(replacement));
        assert_eq!(pins.pinned_variant_count(), 1);
    }

    #[test]
    #[should_panic(expected = "pipeline variant pin count must exist before unpin")]
    fn unpin_rejects_an_unbalanced_cache_release() {
        PipelineVariantPinCounts::default().unpin(variant(19));
    }

    #[test]
    fn cached_command_owner_updates_pins_without_a_second_entry_scan() {
        let source = include_str!("cached_mesh_draw_commands.rs");

        assert!(source.contains("pipeline_variant_pins: PipelineVariantPinCounts"));
        assert!(source.contains("pipeline_variant_pins.pin(variant_id)"));
        assert!(source.contains(".replace(previous.payload.pipeline_variant_id, variant_id)"));
        assert!(source.contains("pipeline_variant_pins.unpin(entry.payload.pipeline_variant_id)"));
        assert!(source.contains("pipeline_variant_pins.clear()"));
        assert!(!source.contains("collect::<HashSet<MeshPipelineVariantId>>"));
    }
}
