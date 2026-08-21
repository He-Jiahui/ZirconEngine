use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{
    component::UiSlotSchema,
    template::{UiChildMount, UiNodeDefinition, UiNodeDefinitionKind},
};

pub(crate) fn native_node_accepts_children(node: &UiNodeDefinition) -> bool {
    native_slot_schemas(node).is_some_and(|slots| {
        slots
            .iter()
            .any(|slot| native_slot_is_available(slot, &node.children))
    })
}

pub(crate) fn default_native_mount(node: &UiNodeDefinition) -> Option<String> {
    native_slot_schemas(node).and_then(|slots| {
        slots
            .iter()
            .find(|slot| native_slot_is_available(slot, &node.children))
            .map(|slot| slot.name.clone())
    })
}

fn native_slot_schemas(node: &UiNodeDefinition) -> Option<&'static [UiSlotSchema]> {
    if !matches!(node.kind, UiNodeDefinitionKind::Native) {
        return None;
    }
    let widget_type = node.widget_type.as_deref()?;
    let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
    registry
        .descriptor(widget_type)
        .map(|descriptor| descriptor.slot_schema.as_slice())
}

fn native_slot_is_available(slot: &UiSlotSchema, children: &[UiChildMount]) -> bool {
    slot.multiple
        || !children
            .iter()
            .any(|child| child.mount.as_deref().unwrap_or_default() == slot.name.as_str())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    fn native_node(widget_type: &str) -> UiNodeDefinition {
        UiNodeDefinition {
            node_id: "native-slot-test".to_string(),
            kind: UiNodeDefinitionKind::Native,
            widget_type: Some(widget_type.to_string()),
            ..UiNodeDefinition::default()
        }
    }

    fn legacy_available_slots(slots: &[UiSlotSchema], children: &[UiChildMount]) -> Vec<String> {
        let mut counts = BTreeMap::<&str, usize>::new();
        for child in children {
            let slot_name = child.mount.as_deref().unwrap_or_default();
            let entry = counts.entry(slot_name).or_insert(0);
            *entry += 1;
        }

        slots
            .iter()
            .filter_map(|slot| {
                let occupied = counts.get(slot.name.as_str()).copied().unwrap_or_default();
                (slot.multiple || occupied == 0).then(|| slot.name.clone())
            })
            .collect()
    }

    #[test]
    fn direct_native_slot_admission_matches_the_legacy_collector() {
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();

        for descriptor in registry.descriptors() {
            let mut node = native_node(&descriptor.id);
            for occupied_slot_count in 0..=descriptor.slot_schema.len() {
                let legacy = legacy_available_slots(&descriptor.slot_schema, &node.children);
                assert_eq!(native_node_accepts_children(&node), !legacy.is_empty());
                assert_eq!(default_native_mount(&node), legacy.first().cloned());

                if let Some(slot) = descriptor.slot_schema.get(occupied_slot_count) {
                    node.children.push(UiChildMount {
                        mount: Some(slot.name.clone()),
                        ..UiChildMount::default()
                    });
                }
            }
        }
    }

    #[test]
    #[ignore = "release-only native slot admission benchmark"]
    fn native_slot_admission_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 10_000;

        fn measure_legacy(node: &UiNodeDefinition, slots: &[UiSlotSchema]) -> u128 {
            let started = Instant::now();
            for _ in 0..CHECKS_PER_SAMPLE {
                black_box(!legacy_available_slots(slots, &node.children).is_empty());
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(node: &UiNodeDefinition) -> u128 {
            let started = Instant::now();
            for _ in 0..CHECKS_PER_SAMPLE {
                black_box(native_node_accepts_children(black_box(node)));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let node = native_node("PropertyRow");
        let descriptor = UiComponentDescriptorRegistry::editor_showcase_shared()
            .descriptor("PropertyRow")
            .expect("PropertyRow descriptor");
        assert!(descriptor.slot_schema.len() >= 2);

        for _ in 0..4 {
            black_box(measure_legacy(&node, &descriptor.slot_schema));
            black_box(measure_optimized(&node));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&node, &descriptor.slot_schema));
                optimized_samples.push(measure_optimized(&node));
            } else {
                optimized_samples.push(measure_optimized(&node));
                legacy_samples.push(measure_legacy(&node, &descriptor.slot_schema));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "RUNTIME75_NATIVE_SLOT_ADMISSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} slot_count={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_map_instances_per_sample={CHECKS_PER_SAMPLE} \
legacy_vec_instances_per_sample={CHECKS_PER_SAMPLE} \
optimized_collection_instances_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            descriptor.slot_schema.len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "direct native-slot admission must reduce P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
