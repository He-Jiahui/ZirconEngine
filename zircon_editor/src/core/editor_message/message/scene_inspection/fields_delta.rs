use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

use super::SceneInspectionPropertyPath;

/// Focused-inspector change identities; field values stay in the runtime artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionFieldsDelta {
    entity: Option<EntityId>,
    requires_resync: bool,
    changed_properties: Arc<[SceneInspectionPropertyPath]>,
    removed_properties: Arc<[SceneInspectionPropertyPath]>,
}

impl SceneInspectionFieldsDelta {
    pub fn unchanged(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: false,
            changed_properties: Vec::new().into(),
            removed_properties: Vec::new().into(),
        }
    }

    pub fn delta(
        entity: EntityId,
        changed_properties: Vec<SceneInspectionPropertyPath>,
        removed_properties: Vec<SceneInspectionPropertyPath>,
    ) -> Self {
        Self {
            entity: Some(entity),
            requires_resync: false,
            changed_properties: changed_properties.into(),
            removed_properties: removed_properties.into(),
        }
    }

    /// Selection changed or the consumer fell behind, so it must read the focused artifact again.
    pub fn resync(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: true,
            changed_properties: Vec::new().into(),
            removed_properties: Vec::new().into(),
        }
    }

    pub const fn entity(&self) -> Option<EntityId> {
        self.entity
    }

    pub const fn requires_resync(&self) -> bool {
        self.requires_resync
    }

    pub fn changed_properties(&self) -> &[SceneInspectionPropertyPath] {
        &self.changed_properties
    }

    pub fn removed_properties(&self) -> &[SceneInspectionPropertyPath] {
        &self.removed_properties
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_du_fields_delta_clone_shares_property_storage() {
        let delta = SceneInspectionFieldsDelta::delta(
            7,
            vec![SceneInspectionPropertyPath::new(
                "gameplay::Transform",
                "translation",
            )],
            vec![SceneInspectionPropertyPath::new(
                "render::Material",
                "roughness",
            )],
        );

        let cloned = delta.clone();

        assert_eq!(
            delta.changed_properties().as_ptr(),
            cloned.changed_properties().as_ptr()
        );
        assert_eq!(
            delta.removed_properties().as_ptr(),
            cloned.removed_properties().as_ptr()
        );
        assert_eq!(delta, cloned);
    }

    #[test]
    fn optimization_batch_du_fields_delta_uses_shared_property_payloads() {
        let production = include_str!("fields_delta.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("scene fields delta production source");

        assert!(production.contains("changed_properties: Arc<[SceneInspectionPropertyPath]>"));
        assert!(production.contains("removed_properties: Arc<[SceneInspectionPropertyPath]>"));
        assert!(production.contains("changed_properties: changed_properties.into()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_du_fields_delta_shared_properties_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const CLONES_PER_SAMPLE: usize = 1_024;
        const PROPERTIES_PER_DELTA: usize = 512;

        let properties = (0..PROPERTIES_PER_DELTA)
            .map(|index| {
                SceneInspectionPropertyPath::new(
                    format!("gameplay::component::Component{index:04}"),
                    format!("field_{index:04}"),
                )
            })
            .collect::<Vec<_>>();
        let shared: Arc<[SceneInspectionPropertyPath]> = properties.clone().into();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_clones(
                    &properties,
                    &shared,
                    CLONES_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_clones(
                    &properties,
                    &shared,
                    CLONES_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_clones(
                    &properties,
                    &shared,
                    CLONES_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_clones(
                    &properties,
                    &shared,
                    CLONES_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR357_SCENE_FIELDS_SHARED_DELTA_BENCH_V1 clones_per_sample={CLONES_PER_SAMPLE} properties_per_delta={PROPERTIES_PER_DELTA} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "scene fields shared delta p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_clones(
            properties: &[SceneInspectionPropertyPath],
            shared: &Arc<[SceneInspectionPropertyPath]>,
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
                    let cloned = properties.to_vec();
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
