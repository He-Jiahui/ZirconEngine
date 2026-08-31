use super::{ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    pub fn remove_instance(&mut self, instance_id: &ViewInstanceId) -> Option<ViewInstance> {
        let removed = self.instances.remove(instance_id)?;
        if self
            .single_instance_index
            .get(&removed.descriptor_id)
            .is_some_and(|current| current == instance_id)
        {
            self.single_instance_index.remove(&removed.descriptor_id);
        }
        Some(removed)
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.single_instance_index.clear();
        self.counters.clear();
    }

    pub(super) fn update_counter(&mut self, instance: &ViewInstance) {
        let Some((_, suffix)) = instance.instance_id.0.rsplit_once('#') else {
            return;
        };
        let Ok(value) = suffix.parse::<usize>() else {
            return;
        };
        if let Some(counter) = self.counters.get_mut(&instance.descriptor_id) {
            *counter = (*counter).max(value);
            return;
        }
        self.counters.insert(instance.descriptor_id.clone(), value);
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use serde_json::Value;

    use super::*;
    use crate::ui::workbench::layout::MainPageId;
    use crate::ui::workbench::view::{ViewDescriptorId, ViewHost};

    #[test]
    fn optimization_batch_dy_existing_view_counter_preserves_maximum_suffix() {
        let descriptor_id = ViewDescriptorId::new("editor.scene");
        let mut registry = ViewRegistry::default();
        for suffix in [2, 9, 4] {
            registry.update_counter(&view_instance(&descriptor_id, suffix));
        }

        assert_eq!(registry.counters.get(&descriptor_id), Some(&9));
        assert_eq!(registry.counters.len(), 1);
    }

    #[test]
    fn optimization_batch_dy_existing_view_counter_borrows_descriptor_id() {
        let production = include_str!("view_registry_instance_mutation.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("view registry instance mutation production source");
        let counter_update = production
            .split("pub(super) fn update_counter")
            .nth(1)
            .expect("view counter update");
        let borrowed_probe = counter_update
            .find(".get_mut(&instance.descriptor_id)")
            .expect("borrowed descriptor probe");
        let owned_insert = counter_update
            .find(".insert(instance.descriptor_id.clone(), value)")
            .expect("owned descriptor insertion");

        assert!(borrowed_probe < owned_insert);
        assert!(!counter_update.contains(".entry(instance.descriptor_id.clone())"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dy_existing_view_counter_update_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const UPDATES_PER_SAMPLE: usize = 16;
        const INSTANCE_COUNT: usize = 1_024;

        let descriptor_id = ViewDescriptorId::new(format!(
            "editor.benchmark.{}",
            "long_repeated_view_descriptor/".repeat(32)
        ));
        let instances = (1..=INSTANCE_COUNT)
            .map(|suffix| view_instance(&descriptor_id, suffix))
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_counter_updates(
                    &instances,
                    UPDATES_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_counter_updates(
                    &instances,
                    UPDATES_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_counter_updates(
                    &instances,
                    UPDATES_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_counter_updates(
                    &instances,
                    UPDATES_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR361_EXISTING_VIEW_COUNTER_UPDATE_BENCH_V1 updates_per_sample={UPDATES_PER_SAMPLE} instance_count={INSTANCE_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "existing view counter update p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn view_instance(descriptor_id: &ViewDescriptorId, suffix: usize) -> ViewInstance {
        ViewInstance {
            instance_id: ViewInstanceId::new(format!("editor.benchmark#{suffix}")),
            descriptor_id: descriptor_id.clone(),
            title: String::new(),
            serializable_payload: Value::Null,
            dirty: false,
            host: ViewHost::ExclusivePage(MainPageId::workbench()),
        }
    }

    fn measure_counter_updates(
        instances: &[ViewInstance],
        update_count: usize,
        optimized: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..update_count {
            let mut registry = ViewRegistry::default();
            for instance in instances {
                if optimized {
                    registry.update_counter(instance);
                } else {
                    legacy_update_counter(&mut registry, instance);
                }
            }
            checksum = checksum.wrapping_add(registry.counters.values().copied().sum::<usize>());
            black_box(registry);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn legacy_update_counter(registry: &mut ViewRegistry, instance: &ViewInstance) {
        let Some((_, suffix)) = instance.instance_id.0.rsplit_once('#') else {
            return;
        };
        let Ok(value) = suffix.parse::<usize>() else {
            return;
        };
        let counter = registry
            .counters
            .entry(instance.descriptor_id.clone())
            .or_insert(0);
        *counter = (*counter).max(value);
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
