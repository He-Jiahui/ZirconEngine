use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

use super::{
    SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionSelectionDelta,
};

/// Runtime-scene change notification without a copied hierarchy or inspector snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionMessage {
    previous_generation: Option<u64>,
    generation: u64,
    focused_entity: Option<EntityId>,
    added_anchors: Arc<[SceneInspectionHierarchyAnchor]>,
    changed_anchors: Arc<[SceneInspectionHierarchyAnchor]>,
    removed_entities: Arc<[EntityId]>,
    hierarchy_reflow_required: bool,
    focused_fields: SceneInspectionFieldsDelta,
    selection: SceneInspectionSelectionDelta,
}

impl SceneInspectionMessage {
    pub fn delta(
        previous_generation: u64,
        generation: u64,
        focused_entity: Option<EntityId>,
        added_anchors: Vec<SceneInspectionHierarchyAnchor>,
        changed_anchors: Vec<SceneInspectionHierarchyAnchor>,
        removed_entities: Vec<EntityId>,
        hierarchy_reflow_required: bool,
        focused_fields: SceneInspectionFieldsDelta,
        selection: SceneInspectionSelectionDelta,
    ) -> Self {
        Self {
            previous_generation: Some(previous_generation),
            generation,
            focused_entity,
            added_anchors: added_anchors.into(),
            changed_anchors: changed_anchors.into(),
            removed_entities: removed_entities.into(),
            hierarchy_reflow_required,
            focused_fields,
            selection,
        }
    }

    /// The receiver has no compatible base generation and must read the runtime artifact anew.
    pub fn resync(generation: u64, focused_entity: Option<EntityId>) -> Self {
        Self::resync_with_selection_revision(generation, focused_entity, 0)
    }

    pub fn resync_with_selection_revision(
        generation: u64,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
    ) -> Self {
        Self {
            previous_generation: None,
            generation,
            focused_entity,
            added_anchors: Vec::new().into(),
            changed_anchors: Vec::new().into(),
            removed_entities: Vec::new().into(),
            hierarchy_reflow_required: true,
            focused_fields: SceneInspectionFieldsDelta::resync(focused_entity),
            selection: SceneInspectionSelectionDelta::resync_at(selection_revision),
        }
    }

    pub(in crate::core::editor_message) fn coalesce_selection_from(&mut self, previous: &Self) {
        self.selection.coalesce_from(&previous.selection);
    }

    pub(crate) fn with_selection_resync_at(mut self, selection_revision: u64) -> Self {
        self.selection = SceneInspectionSelectionDelta::resync_at(selection_revision);
        self
    }

    pub const fn previous_generation(&self) -> Option<u64> {
        self.previous_generation
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn focused_entity(&self) -> Option<EntityId> {
        self.focused_entity
    }

    pub fn added_anchors(&self) -> &[SceneInspectionHierarchyAnchor] {
        &self.added_anchors
    }

    pub fn changed_anchors(&self) -> &[SceneInspectionHierarchyAnchor] {
        &self.changed_anchors
    }

    pub fn removed_entities(&self) -> &[EntityId] {
        &self.removed_entities
    }

    /// The producer rebuilt the hierarchy, so this message cannot be applied as a sparse patch.
    pub const fn requires_hierarchy_reflow(&self) -> bool {
        self.hierarchy_reflow_required
    }

    pub fn focused_fields(&self) -> &SceneInspectionFieldsDelta {
        &self.focused_fields
    }

    pub fn selection(&self) -> &SceneInspectionSelectionDelta {
        &self.selection
    }

    pub const fn requires_resync(&self) -> bool {
        self.previous_generation.is_none()
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use super::{
        SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionMessage,
        SceneInspectionSelectionDelta,
    };

    #[test]
    fn hierarchy_delta_retains_entity_anchor_and_selection_overlay() {
        let added = SceneInspectionHierarchyAnchor::new(7, Some(3), 2, 0xabc);
        let changed = SceneInspectionHierarchyAnchor::new(3, None, 0, 0xdef);
        let selection = SceneInspectionSelectionDelta::delta(vec![7], vec![9]);
        let message = SceneInspectionMessage::delta(
            10,
            11,
            Some(7),
            vec![added.clone()],
            vec![changed.clone()],
            vec![9],
            false,
            SceneInspectionFieldsDelta::unchanged(Some(7)),
            selection,
        );

        assert_eq!(message.added_anchors(), &[added]);
        assert_eq!(message.changed_anchors(), &[changed]);
        assert_eq!(message.removed_entities(), &[9]);
        assert!(!message.requires_hierarchy_reflow());
        assert_eq!(message.selection().added_entities(), &[7]);
        assert_eq!(message.selection().removed_entities(), &[9]);
    }

    #[test]
    fn optimization_batch_dt_scene_message_clone_shares_hierarchy_storage() {
        let message = SceneInspectionMessage::delta(
            41,
            42,
            Some(7),
            vec![SceneInspectionHierarchyAnchor::new(7, Some(3), 2, 0xabc)],
            vec![SceneInspectionHierarchyAnchor::new(3, None, 0, 0xdef)],
            vec![9, 11],
            false,
            SceneInspectionFieldsDelta::unchanged(Some(7)),
            SceneInspectionSelectionDelta::unchanged(),
        );

        let cloned = message.clone();

        assert_eq!(
            message.added_anchors().as_ptr(),
            cloned.added_anchors().as_ptr()
        );
        assert_eq!(
            message.changed_anchors().as_ptr(),
            cloned.changed_anchors().as_ptr()
        );
        assert_eq!(
            message.removed_entities().as_ptr(),
            cloned.removed_entities().as_ptr()
        );
        assert_eq!(message, cloned);
    }

    #[test]
    fn optimization_batch_dt_scene_message_uses_shared_hierarchy_payloads() {
        let production = include_str!("message.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("scene inspection message production source");

        assert!(production.contains("added_anchors: Arc<[SceneInspectionHierarchyAnchor]>"));
        assert!(production.contains("changed_anchors: Arc<[SceneInspectionHierarchyAnchor]>"));
        assert!(production.contains("removed_entities: Arc<[EntityId]>"));
        assert!(production.contains("added_anchors: added_anchors.into()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dt_scene_message_shared_hierarchy_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const CLONES_PER_SAMPLE: usize = 4_096;
        const ANCHORS_PER_MESSAGE: usize = 512;

        let anchors = (0..ANCHORS_PER_MESSAGE)
            .map(|index| {
                SceneInspectionHierarchyAnchor::new(
                    index as u64,
                    index.checked_sub(1).map(|parent| parent as u64),
                    index as u32,
                    index as u64 * 17,
                )
            })
            .collect::<Vec<_>>();
        let shared: Arc<[SceneInspectionHierarchyAnchor]> = anchors.clone().into();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_clones(&anchors, &shared, CLONES_PER_SAMPLE, false));
                optimized_samples.push(measure_clones(&anchors, &shared, CLONES_PER_SAMPLE, true));
            } else {
                optimized_samples.push(measure_clones(&anchors, &shared, CLONES_PER_SAMPLE, true));
                legacy_samples.push(measure_clones(&anchors, &shared, CLONES_PER_SAMPLE, false));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR356_SCENE_MESSAGE_SHARED_HIERARCHY_BENCH_V1 clones_per_sample={CLONES_PER_SAMPLE} anchors_per_message={ANCHORS_PER_MESSAGE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "scene message shared hierarchy p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_clones(
            anchors: &[SceneInspectionHierarchyAnchor],
            shared: &Arc<[SceneInspectionHierarchyAnchor]>,
            clone_count: usize,
            optimized: bool,
        ) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for _ in 0..clone_count {
                if optimized {
                    let cloned = Arc::clone(shared);
                    checksum = checksum.wrapping_add(cloned.len());
                    black_box(cloned);
                } else {
                    let cloned = anchors.to_vec();
                    checksum = checksum.wrapping_add(cloned.len());
                    black_box(cloned);
                }
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
