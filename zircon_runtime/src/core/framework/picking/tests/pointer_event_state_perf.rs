use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::RenderViewportHandle;

use super::*;

fn pointer_location(pointer: u64, x: f32) -> PointerLocation {
    PointerLocation::new(
        PointerId::new(pointer),
        RenderViewportHandle::new(3),
        Vec2::new(x, x + 1.0),
    )
}

#[test]
fn runtime47_batch_location_map_preserves_input_override() {
    let initial = pointer_location(7, 10.0);
    let other = pointer_location(9, 20.0);
    let override_location = pointer_location(7, 30.0);
    let input = PointerInput::new(
        override_location,
        PointerAction::Move {
            delta: Vec2::new(20.0, 20.0),
        },
    );

    let locations = location_map(&[initial, other], &[input]);

    assert_eq!(locations.len(), 2);
    assert_eq!(locations.get(&PointerId::new(7)), Some(&override_location));
    assert_eq!(locations.get(&PointerId::new(9)), Some(&other));
}

#[test]
fn runtime47_batch_location_map_uses_capacity_hash_index() {
    let source = include_str!("../pointer_event_state.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("pointer event production source");
    let location_map = production
        .split("fn location_map")
        .nth(1)
        .expect("location map")
        .split("mod optimization_tests")
        .next()
        .expect("bounded location map");

    assert!(location_map.contains("HashMap<PointerId, PointerLocation>"));
    assert!(location_map.contains("HashMap::with_capacity"));
    assert!(!location_map.contains("collect::<BTreeMap"));
    assert_eq!(
        production
            .matches("&HashMap<PointerId, PointerLocation>")
            .count(),
        2
    );
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn runtime47_batch_pointer_location_hash_map_performance_evidence() {
    fn legacy_location_map(
        pointer_locations: &[PointerLocation],
        inputs: &[PointerInput],
    ) -> BTreeMap<PointerId, PointerLocation> {
        let mut locations = pointer_locations
            .iter()
            .copied()
            .map(|location| (location.pointer, location))
            .collect::<BTreeMap<_, _>>();
        for input in inputs {
            locations.insert(input.pointer(), input.location);
        }
        locations
    }

    let pointer_locations = (0..32_768_u64)
        .map(|index| pointer_location(index, index as f32))
        .collect::<Vec<_>>();
    let inputs = (0..16_384_u64)
        .map(|index| {
            let location = pointer_location(index * 2, index as f32 + 0.5);
            PointerInput::new(
                location,
                PointerAction::Move {
                    delta: Vec2::new(0.5, 0.5),
                },
            )
        })
        .collect::<Vec<_>>();
    const SAMPLE_PAIRS: usize = 17;
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_location_map(
            black_box(&pointer_locations),
            black_box(&inputs),
        ));
        started.elapsed().as_nanos().max(1)
    };
    let measure_hash = || {
        let started = Instant::now();
        black_box(location_map(
            black_box(&pointer_locations),
            black_box(&inputs),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_hash());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            hash_samples.push(measure_hash());
        } else {
            hash_samples.push(measure_hash());
            legacy_samples.push(measure_legacy());
        }
    }

    legacy_samples.sort_unstable();
    hash_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let hash_p50 = hash_samples[8];
    let hash_p95 = hash_samples[16];
    println!(
        "RUNTIME47_POINTER_LOCATION_HASH_MAP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 hash_first_pairs=8 pointer_locations={} input_overrides={} legacy_p50_ns={} legacy_p95_ns={} hash_p50_ns={} hash_p95_ns={} legacy_tree_writes={} hash_writes={} target_ratio_bp=6000",
        pointer_locations.len(),
        inputs.len(),
        legacy_p50,
        legacy_p95,
        hash_p50,
        hash_p95,
        pointer_locations.len() + inputs.len(),
        pointer_locations.len() + inputs.len(),
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "pointer location HashMap P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}
