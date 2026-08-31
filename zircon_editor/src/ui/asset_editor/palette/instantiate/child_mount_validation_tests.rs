use std::collections::{BTreeMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::{
    UiChildMount, UiComponentDefinition, UiNamedSlotSchema,
};

use super::validate_child_mounts_for_component;

fn child(slot: &str) -> UiChildMount {
    UiChildMount {
        mount: Some(slot.to_string()),
        ..UiChildMount::default()
    }
}

fn component() -> UiComponentDefinition {
    UiComponentDefinition {
        slots: BTreeMap::from([
            (
                "header".to_string(),
                UiNamedSlotSchema {
                    required: true,
                    multiple: false,
                    ..UiNamedSlotSchema::default()
                },
            ),
            (
                "items".to_string(),
                UiNamedSlotSchema {
                    multiple: true,
                    ..UiNamedSlotSchema::default()
                },
            ),
        ]),
        ..UiComponentDefinition::default()
    }
}

#[test]
fn borrowed_mount_set_preserves_required_single_and_multiple_semantics() {
    let component = component();

    assert!(validate_child_mounts_for_component(
        &[child("header"), child("items"), child("items")],
        &component,
    )
    .is_some());
    assert!(
        validate_child_mounts_for_component(&[child("header"), child("header")], &component)
            .is_none()
    );
    assert!(validate_child_mounts_for_component(&[child("items")], &component).is_none());
    assert!(
        validate_child_mounts_for_component(&[child("header"), child("unknown")], &component)
            .is_none()
    );
}

#[test]
#[ignore = "release-only palette child-mount validation benchmark"]
fn palette_child_mount_set_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 10_000;
    const SLOT_COUNT: usize = 32;
    const CHILD_COUNT: usize = 64;

    fn fixture() -> (UiComponentDefinition, Vec<UiChildMount>) {
        let slots = (0..SLOT_COUNT)
            .map(|index| {
                (
                    format!("slot-{index:02}"),
                    UiNamedSlotSchema {
                        required: true,
                        multiple: true,
                        ..UiNamedSlotSchema::default()
                    },
                )
            })
            .collect();
        let children = (0..CHILD_COUNT)
            .map(|index| child(&format!("slot-{:02}", index % SLOT_COUNT)))
            .collect();
        (
            UiComponentDefinition {
                slots,
                ..UiComponentDefinition::default()
            },
            children,
        )
    }

    fn legacy(children: &[UiChildMount], component: &UiComponentDefinition) -> Option<()> {
        let mut counts = BTreeMap::<&str, usize>::new();
        for child in children {
            let slot_name = child.mount.as_deref().unwrap_or_default();
            let slot = component.slots.get(slot_name)?;
            let count = counts.entry(slot_name).or_insert(0);
            *count += 1;
            if !slot.multiple && *count > 1 {
                return None;
            }
        }
        component
            .slots
            .iter()
            .all(|(slot_name, slot)| !slot.required || counts.contains_key(slot_name.as_str()))
            .then_some(())
    }

    fn measure(
        children: &[UiChildMount],
        component: &UiComponentDefinition,
        validate: fn(&[UiChildMount], &UiComponentDefinition) -> Option<()>,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(validate(black_box(children), black_box(component)));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    let (component, children) = fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&children, &component, legacy));
            optimized_samples.push(measure(
                &children,
                &component,
                validate_child_mounts_for_component,
            ));
        } else {
            optimized_samples.push(measure(
                &children,
                &component,
                validate_child_mounts_for_component,
            ));
            legacy_samples.push(measure(&children, &component, legacy));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "RUNTIME75_PALETTE_CHILD_MOUNT_SET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} slot_count={SLOT_COUNT} child_count={CHILD_COUNT} legacy_tree_nodes_per_check={SLOT_COUNT} optimized_preallocated_sets_per_check=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={legacy_samples:?} optimized_raw_ns={optimized_samples:?}"
    );

    assert_eq!(
        validate_child_mounts_for_component(&children, &component),
        legacy(&children, &component)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "preallocated child-mount sets must reduce P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
