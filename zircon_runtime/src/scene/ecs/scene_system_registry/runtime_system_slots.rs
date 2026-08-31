use std::collections::HashMap;

use super::{BoxedRuntimeSceneSystem, SystemStage};

#[derive(Default)]
pub(super) struct RuntimeSystemSlots {
    slots: Vec<RuntimeSystemSlot>,
    indices: HashMap<String, usize>,
}

impl RuntimeSystemSlots {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn contains(&self, id: &str) -> bool {
        self.indices.contains_key(id)
    }

    pub(super) fn iter(&self) -> RuntimeSystems<'_> {
        RuntimeSystems {
            slots: self.slots.iter(),
        }
    }

    pub(super) fn take(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let index = *self.indices.get(id)?;
        self.slots.get_mut(index)?.system.take()
    }

    pub(super) fn remove(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let index = *self.indices.get(id)?;
        let slot = self.slots.remove(index);
        self.indices.remove(id);
        self.rebuild_indices_from(index);
        slot.system
    }

    pub(super) fn restore(&mut self, system: BoxedRuntimeSceneSystem) {
        let id = system.id();
        let index = *self
            .indices
            .get(id)
            .expect("taken runtime system must retain its registry slot");
        let slot = &mut self.slots[index];
        debug_assert!(slot.system.is_none());
        debug_assert_eq!(slot.id, id);
        slot.system = Some(system);
    }

    pub(super) fn insert(&mut self, system: BoxedRuntimeSceneSystem) {
        let slot = RuntimeSystemSlot::new(system);
        let insert_index = match self
            .slots
            .binary_search_by(|existing| compare_slots(existing, &slot))
        {
            Ok(index) | Err(index) => index,
        };
        let id = slot.id.clone();
        self.slots.insert(insert_index, slot);
        self.indices.insert(id, insert_index);
        self.rebuild_indices_from(insert_index + 1);
    }

    fn rebuild_indices_from(&mut self, start: usize) {
        for (index, slot) in self.slots.iter().enumerate().skip(start) {
            self.indices.insert(slot.id.clone(), index);
        }
    }
}

struct RuntimeSystemSlot {
    id: String,
    stage: SystemStage,
    order: i32,
    system: Option<BoxedRuntimeSceneSystem>,
}

impl RuntimeSystemSlot {
    fn new(system: BoxedRuntimeSceneSystem) -> Self {
        Self {
            id: system.id().to_owned(),
            stage: system.stage(),
            order: system.order(),
            system: Some(system),
        }
    }
}

fn compare_slots(left: &RuntimeSystemSlot, right: &RuntimeSystemSlot) -> std::cmp::Ordering {
    left.stage
        .rank()
        .cmp(&right.stage.rank())
        .then(left.order.cmp(&right.order))
        .then(left.id.cmp(&right.id))
}

pub(crate) struct RuntimeSystems<'registry> {
    slots: std::slice::Iter<'registry, RuntimeSystemSlot>,
}

impl RuntimeSystems<'_> {
    pub(crate) fn iter(&self) -> Self {
        Self {
            slots: self.slots.clone(),
        }
    }
}

impl<'registry> Iterator for RuntimeSystems<'registry> {
    type Item = &'registry BoxedRuntimeSceneSystem;

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            if let Some(system) = slot.system.as_ref() {
                return Some(system);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::scene::ecs::{
        FunctionRuntimeSceneSystem, Schedule, SystemOrderingConstraint, SystemRef,
    };
    use std::{hint::black_box, time::Instant};

    #[test]
    fn runtime_slots_keep_indices_stable_across_holes_restore_and_cold_removal() {
        let mut registry = SceneSystemRegistry::new();
        for (id, order) in [("runtime.c", 30), ("runtime.a", 10), ("runtime.b", 20)] {
            registry
                .register_boxed_runtime_system(runtime_system(id, order))
                .expect("runtime system should register");
        }
        assert_eq!(
            runtime_ids(&registry),
            ["runtime.a", "runtime.b", "runtime.c"]
        );

        let system_b = registry
            .take_runtime_system("runtime.b")
            .expect("middle slot should be indexed");
        let system_a = registry
            .take_runtime_system("runtime.a")
            .expect("first slot should remain indexed with a middle hole");
        let system_c = registry
            .take_runtime_system("runtime.c")
            .expect("last slot should remain indexed with two holes");
        assert!(registry.take_runtime_system("runtime.b").is_none());
        assert!(matches!(
            registry.register_boxed_runtime_system(runtime_system("runtime.b", 20)),
            Err(ScheduleError::DuplicateSystem(id)) if id == "runtime.b"
        ));

        registry.restore_runtime_system(system_c);
        registry.restore_runtime_system(system_a);
        registry.restore_runtime_system(system_b);
        assert_eq!(
            runtime_ids(&registry),
            ["runtime.a", "runtime.b", "runtime.c"]
        );

        let removed = registry
            .remove_runtime_system("runtime.b")
            .expect("middle system should be removed on the cold path");
        assert_eq!(removed.id(), "runtime.b");
        let system_c = registry
            .take_runtime_system("runtime.c")
            .expect("shifted last slot should receive a rebuilt index");
        registry.restore_runtime_system(system_c);
        registry
            .register_boxed_runtime_system(runtime_system("runtime.b", 20))
            .expect("middle system should reinsert into sorted order");
        assert_eq!(
            runtime_ids(&registry),
            ["runtime.a", "runtime.b", "runtime.c"]
        );
    }

    #[test]
    fn failed_schedule_registration_removes_slot_and_preserves_existing_index() {
        let mut schedule = Schedule::default();
        let system_a = runtime_system_with_constraint(
            "runtime.rollback.a",
            10,
            SystemOrderingConstraint::Before(SystemRef::System("runtime.rollback.b".to_string())),
        );
        schedule
            .register_boxed_runtime_system(system_a)
            .expect("missing constraint target should defer the edge");

        let system_b = runtime_system_with_constraint(
            "runtime.rollback.b",
            20,
            SystemOrderingConstraint::Before(SystemRef::System("runtime.rollback.a".to_string())),
        );
        assert!(matches!(
            schedule.register_boxed_runtime_system(system_b),
            Err(ScheduleError::OrderingCycle { .. })
        ));

        assert!(schedule.take_runtime_system("runtime.rollback.b").is_none());
        let system_a = schedule
            .take_runtime_system("runtime.rollback.a")
            .expect("rollback must preserve the surviving runtime index");
        schedule.restore_runtime_system(system_a);
        assert!(schedule.take_runtime_system("runtime.rollback.a").is_some());
    }

    #[test]
    #[ignore = "release profiling gate; run explicitly with --include-ignored --nocapture"]
    fn runtime22_performance_runtime_system_slot_index_profile() {
        const SYSTEM_COUNT: usize = 1_000;
        const WARMUP_PAIRS: usize = 10;
        const SAMPLE_PAIRS: usize = 31;
        const MAX_INDEXED_P95_PERCENT: u128 = 50;

        assert!(
            !cfg!(debug_assertions),
            "runtime-system slot performance evidence requires a release test binary"
        );

        let ids: Vec<_> = (0..SYSTEM_COUNT)
            .map(|index| format!("runtime.profile.{index:04}"))
            .collect();
        let mut legacy_systems: Vec<_> = ids
            .iter()
            .enumerate()
            .map(|(index, id)| runtime_system(id, index as i32))
            .collect();
        let mut indexed_registry = SceneSystemRegistry::new();
        for (index, id) in ids.iter().enumerate() {
            indexed_registry
                .register_boxed_runtime_system(runtime_system(id, index as i32))
                .expect("profile runtime system should register");
        }

        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut indexed_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
            let indexed_first = pair_index % 2 == 1;
            let (legacy_elapsed, indexed_elapsed) = if indexed_first {
                let indexed = profile_indexed_cycle(&mut indexed_registry, &ids);
                let legacy = profile_legacy_cycle(&mut legacy_systems, &ids);
                (legacy, indexed)
            } else {
                let legacy = profile_legacy_cycle(&mut legacy_systems, &ids);
                let indexed = profile_indexed_cycle(&mut indexed_registry, &ids);
                (legacy, indexed)
            };
            if pair_index >= WARMUP_PAIRS {
                legacy_samples_ns.push(legacy_elapsed);
                indexed_samples_ns.push(indexed_elapsed);
            }
        }

        legacy_samples_ns.sort_unstable();
        indexed_samples_ns.sort_unstable();
        let legacy_p50_ns = percentile(&legacy_samples_ns, 50);
        let legacy_p95_ns = percentile(&legacy_samples_ns, 95);
        let indexed_p50_ns = percentile(&indexed_samples_ns, 50);
        let indexed_p95_ns = percentile(&indexed_samples_ns, 95);
        let p95_reduction_percent =
            100.0 * (1.0 - indexed_p95_ns as f64 / legacy_p95_ns.max(1) as f64);

        println!(
            "PERF_RESULT runtime22.runtime_system_slots profile=release system_count={SYSTEM_COUNT} sample_pairs={SAMPLE_PAIRS} pair_order=alternating legacy_p50_us={:.3} legacy_p95_us={:.3} indexed_p50_us={:.3} indexed_p95_us={:.3} p95_reduction_percent={p95_reduction_percent:.2} required_reduction_percent=50",
            legacy_p50_ns as f64 / 1_000.0,
            legacy_p95_ns as f64 / 1_000.0,
            indexed_p50_ns as f64 / 1_000.0,
            indexed_p95_ns as f64 / 1_000.0,
        );
        assert!(
            indexed_p95_ns.saturating_mul(100)
                <= legacy_p95_ns.saturating_mul(MAX_INDEXED_P95_PERCENT),
            "indexed slot P95 {indexed_p95_ns}ns must be at most {MAX_INDEXED_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn profile_legacy_cycle(systems: &mut Vec<BoxedRuntimeSceneSystem>, ids: &[String]) -> u128 {
        let started_at = Instant::now();
        for id in ids {
            let system = legacy_take_runtime_system(systems, id)
                .expect("legacy profile system should remain registered");
            black_box(system.id());
            legacy_restore_runtime_system(systems, system);
        }
        started_at.elapsed().as_nanos()
    }

    fn profile_indexed_cycle(registry: &mut SceneSystemRegistry, ids: &[String]) -> u128 {
        let started_at = Instant::now();
        for id in ids {
            let system = registry
                .take_runtime_system(id)
                .expect("indexed profile system should remain registered");
            black_box(system.id());
            registry.restore_runtime_system(system);
        }
        started_at.elapsed().as_nanos()
    }

    fn legacy_take_runtime_system(
        systems: &mut Vec<BoxedRuntimeSceneSystem>,
        id: &str,
    ) -> Option<BoxedRuntimeSceneSystem> {
        let mut index = 0_usize;
        while index < systems.len() {
            if systems[index].id() == id {
                return Some(systems.remove(index));
            }
            index += 1;
        }
        None
    }

    fn legacy_restore_runtime_system(
        systems: &mut Vec<BoxedRuntimeSceneSystem>,
        system: BoxedRuntimeSceneSystem,
    ) {
        let insert_index = match systems.binary_search_by(|existing| {
            existing
                .stage()
                .rank()
                .cmp(&system.stage().rank())
                .then(existing.order().cmp(&system.order()))
                .then(existing.id().cmp(system.id()))
        }) {
            Ok(index) | Err(index) => index,
        };
        systems.insert(insert_index, system);
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        samples[(samples.len() * percentile).div_ceil(100) - 1]
    }

    fn runtime_system(id: &str, order: i32) -> BoxedRuntimeSceneSystem {
        Box::new(FunctionRuntimeSceneSystem::new(
            SceneSystemMetadata::new(id, SystemStage::Update, order),
            |_| Ok(()),
        ))
    }

    fn runtime_system_with_constraint(
        id: &str,
        order: i32,
        constraint: SystemOrderingConstraint,
    ) -> BoxedRuntimeSceneSystem {
        Box::new(FunctionRuntimeSceneSystem::new(
            SceneSystemMetadata::new(id, SystemStage::Update, order).with_constraint(constraint),
            |_| Ok(()),
        ))
    }

    fn runtime_ids(registry: &SceneSystemRegistry) -> Vec<&str> {
        registry
            .runtime_systems()
            .iter()
            .map(|system| system.id())
            .collect()
    }
}
