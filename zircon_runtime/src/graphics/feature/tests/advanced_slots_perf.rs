use std::hint::black_box;

use super::{descriptor_only_advanced_slot, BuiltinRenderFeature, DESCRIPTOR_ONLY_ADVANCED_SLOTS};

const LOOKUP_ROUNDS: usize = 65_536;

#[test]
fn optimization_batch_20260830dl_descriptor_slots_are_sorted_for_binary_lookup() {
    assert!(DESCRIPTOR_ONLY_ADVANCED_SLOTS
        .windows(2)
        .all(|slots| slots[0].feature < slots[1].feature));

    for feature in BuiltinRenderFeature::ALL {
        let legacy = DESCRIPTOR_ONLY_ADVANCED_SLOTS
            .iter()
            .find(|slot| slot.feature == *feature)
            .map(|slot| slot.descriptor_name());
        let optimized = descriptor_only_advanced_slot(*feature).map(|slot| slot.descriptor_name());
        assert_eq!(optimized, legacy, "lookup changed for {feature:?}");
    }
}

#[test]
#[ignore = "deterministic comparator-count evidence for the managed optimization batch"]
fn optimization_batch_20260830dl_descriptor_slot_lookup_evidence() {
    let mut linear_comparisons = 0_u64;
    let mut binary_comparisons = 0_u64;

    for round in 0..LOOKUP_ROUNDS {
        let feature = black_box(BuiltinRenderFeature::ALL[round % BuiltinRenderFeature::ALL.len()]);
        let legacy = DESCRIPTOR_ONLY_ADVANCED_SLOTS.iter().find(|slot| {
            linear_comparisons += 1;
            slot.feature == feature
        });
        let optimized = DESCRIPTOR_ONLY_ADVANCED_SLOTS
            .binary_search_by(|slot| {
                binary_comparisons += 1;
                slot.feature.cmp(&feature)
            })
            .ok()
            .map(|index| &DESCRIPTOR_ONLY_ADVANCED_SLOTS[index]);
        assert_eq!(
            optimized.map(|slot| slot.descriptor_name()),
            legacy.map(|slot| slot.descriptor_name())
        );
    }

    let reduction_basis_points = linear_comparisons
        .saturating_sub(binary_comparisons)
        .saturating_mul(10_000)
        / linear_comparisons;
    println!(
        "RUNTIME523_ADVANCED_SLOT_BINARY_LOOKUP_BENCH_V1 rounds={LOOKUP_ROUNDS} linear_comparisons={linear_comparisons} binary_comparisons={binary_comparisons} comparison_reduction_basis_points={reduction_basis_points}"
    );
    assert!(binary_comparisons < linear_comparisons);
}
