use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

/// Revision-checked editor selection changes for a retained hierarchy projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionSelectionDelta {
    previous_revision: Option<u64>,
    revision: u64,
    added_entities: Vec<EntityId>,
    removed_entities: Vec<EntityId>,
}

impl SceneInspectionSelectionDelta {
    pub fn unchanged() -> Self {
        Self::unchanged_at(0)
    }

    pub fn unchanged_at(revision: u64) -> Self {
        Self {
            previous_revision: Some(revision),
            revision,
            added_entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    pub fn delta(added_entities: Vec<EntityId>, removed_entities: Vec<EntityId>) -> Self {
        Self::between(0, 1, added_entities, removed_entities)
    }

    pub fn between(
        previous_revision: u64,
        revision: u64,
        added_entities: Vec<EntityId>,
        removed_entities: Vec<EntityId>,
    ) -> Self {
        Self {
            previous_revision: Some(previous_revision),
            revision,
            added_entities,
            removed_entities,
        }
    }

    /// The receiving projection has no compatible selection revision.
    pub fn resync() -> Self {
        Self::resync_at(0)
    }

    pub fn resync_at(revision: u64) -> Self {
        Self {
            previous_revision: None,
            revision,
            added_entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    /// Composes a superseded Latest delta into this newer delta.
    pub(super) fn coalesce_from(&mut self, previous: &Self) {
        if previous.requires_resync()
            || self.requires_resync()
            || self.previous_revision != Some(previous.revision)
        {
            *self = Self::resync_at(self.revision);
            return;
        }

        let mut added = HashSet::with_capacity(
            previous
                .added_entities
                .len()
                .saturating_add(self.added_entities.len()),
        );
        added.extend(previous.added_entities.iter().copied());
        let mut removed = HashSet::with_capacity(
            previous
                .removed_entities
                .len()
                .saturating_add(self.removed_entities.len()),
        );
        removed.extend(previous.removed_entities.iter().copied());
        for entity in &self.added_entities {
            if !removed.remove(entity) {
                added.insert(*entity);
            }
        }
        for entity in &self.removed_entities {
            if !added.remove(entity) {
                removed.insert(*entity);
            }
        }
        self.previous_revision = previous.previous_revision;
        self.added_entities = added.into_iter().collect();
        self.added_entities.sort_unstable();
        self.removed_entities = removed.into_iter().collect();
        self.removed_entities.sort_unstable();
    }

    pub const fn previous_revision(&self) -> Option<u64> {
        self.previous_revision
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn requires_resync(&self) -> bool {
        self.previous_revision.is_none()
    }

    pub fn added_entities(&self) -> &[EntityId] {
        &self.added_entities
    }

    pub fn removed_entities(&self) -> &[EntityId] {
        &self.removed_entities
    }
}

#[cfg(test)]
mod tests {
    use super::SceneInspectionSelectionDelta;

    #[test]
    fn latest_deltas_compose_relative_to_the_oldest_retained_revision() {
        let previous = SceneInspectionSelectionDelta::between(4, 5, vec![7, 9], vec![3]);
        let mut current = SceneInspectionSelectionDelta::between(5, 6, vec![3, 11], vec![9]);

        current.coalesce_from(&previous);

        assert_eq!(current.previous_revision(), Some(4));
        assert_eq!(current.revision(), 6);
        assert_eq!(current.added_entities(), &[7, 11]);
        assert!(current.removed_entities().is_empty());
    }

    #[test]
    fn optimization_batch_20260826f_editor48_hash_coalescing_preserves_semantics() {
        let previous =
            SceneInspectionSelectionDelta::between(8, 9, vec![9, 7, 7, 5], vec![4, 2, 2]);
        let mut current =
            SceneInspectionSelectionDelta::between(9, 10, vec![4, 11, 11], vec![9, 5]);

        current.coalesce_from(&previous);

        assert_eq!(current.previous_revision(), Some(8));
        assert_eq!(current.added_entities(), &[7, 11]);
        assert_eq!(current.removed_entities(), &[2]);
    }

    #[test]
    fn optimization_batch_20260826f_editor48_hash_coalescing_uses_hash_accumulation() {
        let source = include_str!("selection_delta.rs");
        let coalescing = source
            .split("pub(super) fn coalesce_from")
            .nth(1)
            .expect("selection coalescing")
            .split("pub const fn previous_revision")
            .next()
            .expect("bounded selection coalescing");

        assert!(source.contains("use std::collections::HashSet;"));
        assert!(coalescing.contains("HashSet::with_capacity"));
        assert!(coalescing.contains("sort_unstable"));
        assert!(!coalescing.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826f_editor48_hash_coalescing_performance_evidence() {
        use std::collections::BTreeSet;
        use std::hint::black_box;
        use std::time::Instant;

        fn legacy_coalesce(
            previous_added: &[u64],
            previous_removed: &[u64],
            current_added: &[u64],
            current_removed: &[u64],
        ) -> (Vec<u64>, Vec<u64>) {
            let mut added = previous_added.iter().copied().collect::<BTreeSet<_>>();
            let mut removed = previous_removed.iter().copied().collect::<BTreeSet<_>>();
            for entity in current_added {
                if !removed.remove(entity) {
                    added.insert(*entity);
                }
            }
            for entity in current_removed {
                if !added.remove(entity) {
                    removed.insert(*entity);
                }
            }
            (added.into_iter().collect(), removed.into_iter().collect())
        }

        let previous_added = (0..32_768_u64).collect::<Vec<_>>();
        let previous =
            SceneInspectionSelectionDelta::between(20, 21, previous_added.clone(), Vec::new());
        let current_template =
            SceneInspectionSelectionDelta::between(21, 22, Vec::new(), previous_added.clone());
        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_coalesce(
                &previous.added_entities,
                &previous.removed_entities,
                &current_template.added_entities,
                &current_template.removed_entities,
            ));
            legacy_samples.push(started.elapsed().as_nanos());

            let mut current = current_template.clone();
            let started = Instant::now();
            current.coalesce_from(black_box(&previous));
            black_box(current);
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "EDITOR48_SCENE_SELECTION_HASH_COALESCING_BENCH_V1 entities={} legacy_p95_ns={} hash_p95_ns={} legacy_tree_updates={} hash_updates={} target_ratio_bp=6000",
            previous_added.len(),
            legacy_p95,
            hash_p95,
            previous_added.len() * 2,
            previous_added.len() * 2,
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "hash selection coalescing P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
