use std::collections::{BTreeSet, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Mat4, Vec3};

const PROBE_COUNT: u64 = 65_536;
const SAMPLE_PAIRS: usize = 21;

fn probe(probe_id: u64, update: PlanarUpdateMode) -> PlanarReflectionProbeData {
    PlanarReflectionProbeData {
        probe_id,
        plane_transform: Mat4::IDENTITY,
        local_reference_position: Vec3::ZERO,
        bounds_min: Vec3::splat(-1.0),
        bounds_max: Vec3::splat(1.0),
        resolution: 256,
        update,
        capture_target: None,
        layer_mask: RenderLayerSet::default(),
    }
}

#[test]
fn optimization_batch_20260826cp_runtime133_single_hash_state_preserves_probe_lifecycle() {
    let mut state = PlanarReflectionUpdateState::default();
    let first = probe(11, PlanarUpdateMode::OnDemand);
    let second = probe(22, PlanarUpdateMode::OnDemand);

    assert!(state.should_capture(&first));
    assert!(state.should_capture(&second));
    state.mark_captured(first.probe_id);
    assert!(!state.should_capture(&first));
    assert!(state.should_capture(&second));

    state.mark_dirty(first.probe_id);
    assert!(state.should_capture(&first));
    state.mark_captured(first.probe_id);
    assert!(!state.should_capture(&first));

    state.forget(first.probe_id);
    assert!(state.should_capture(&first));
    assert!(state.should_capture(&probe(11, PlanarUpdateMode::EveryFrame)));
}

#[test]
fn optimization_batch_20260826cp_runtime133_planar_probe_uses_one_hash_lookup_owner() {
    let source = include_str!("../update_state.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("states: HashMap<u64, PlanarProbeCaptureState>"));
    assert!(source.contains("self.states.get(&probe.probe_id)"));
    assert!(!source.contains("BTreeSet"));
    assert!(!source.contains("captured.contains"));
    assert!(!source.contains("dirty.contains"));
}

fn legacy_cycle() -> usize {
    let mut captured = BTreeSet::new();
    let mut dirty = BTreeSet::new();
    for probe_id in 0..PROBE_COUNT {
        captured.insert(probe_id);
    }
    for probe_id in (0..PROBE_COUNT).step_by(4) {
        dirty.insert(probe_id);
    }
    let mut capture_count = 0;
    for probe_id in 0..PROBE_COUNT {
        capture_count += usize::from(!captured.contains(&probe_id) || dirty.contains(&probe_id));
    }
    for probe_id in dirty {
        captured.insert(probe_id);
    }
    capture_count
}

fn optimized_cycle() -> usize {
    let mut states = HashMap::with_capacity(PROBE_COUNT as usize);
    for probe_id in 0..PROBE_COUNT {
        states.insert(probe_id, PlanarProbeCaptureState::Captured);
    }
    for probe_id in (0..PROBE_COUNT).step_by(4) {
        states.insert(probe_id, PlanarProbeCaptureState::Dirty);
    }
    let mut capture_count = 0;
    for probe_id in 0..PROBE_COUNT {
        capture_count += usize::from(!matches!(
            states.get(&probe_id),
            Some(PlanarProbeCaptureState::Captured)
        ));
    }
    for probe_id in (0..PROBE_COUNT).step_by(4) {
        states.insert(probe_id, PlanarProbeCaptureState::Captured);
    }
    capture_count
}

fn elapsed_ns(run: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(run()), (PROBE_COUNT / 4) as usize);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826cp_runtime133_planar_probe_single_hash_performance_evidence() {
    for _ in 0..3 {
        assert_eq!(black_box(legacy_cycle()), optimized_cycle());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(legacy_cycle));
            optimized_samples.push(elapsed_ns(optimized_cycle));
        } else {
            optimized_samples.push(elapsed_ns(optimized_cycle));
            legacy_samples.push(elapsed_ns(legacy_cycle));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "RUNTIME133_PLANAR_PROBE_SINGLE_HASH_STATE_BENCH_V1 sample_pairs={} probe_count={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        PROBE_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single hash probe state p95 must be at least 30% below dual tree state: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
