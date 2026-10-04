use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::super::declarations::{
    HybridGiRuntimeProbeSceneData, HybridGiRuntimeState, HybridGiRuntimeTraceRegionSceneData,
};
use super::{
    decay_request_support_q8, decay_support_q8, quantize_support_q8,
    ANCESTOR_TRACE_SUPPORT_FALLOFF, DESCENDANT_TRACE_SUPPORT_FALLOFF,
};

const PROBE_COUNT: usize = 4_096;
const TRACE_REGION_COUNT: usize = 16;
const LEGACY_REGION_RESOLUTIONS_PER_REFRESH: usize = 12_286;
const SAMPLE_PAIRS: usize = 21;

fn fixture() -> HybridGiRuntimeState {
    let mut state = HybridGiRuntimeState::default();
    for probe_id in 0..PROBE_COUNT as u32 {
        state.probe_scene_data_mut().insert(
            probe_id,
            HybridGiRuntimeProbeSceneData::new(2_048 + probe_id % 32, 2_048, 2_048, 96),
        );
        if probe_id > 0 {
            state.probe_parent_probes_mut().insert(probe_id, 0);
        }
    }
    state.rebuild_probe_child_probes();

    for region_id in 0..TRACE_REGION_COUNT as u32 {
        state.trace_region_scene_data_mut().insert(
            region_id,
            HybridGiRuntimeTraceRegionSceneData::new(
                2_048 + region_id,
                2_048,
                2_048,
                192,
                128,
                [64, 96, 128],
            ),
        );
    }
    state.assign_scheduled_trace_regions(0..TRACE_REGION_COUNT as u32);
    state
}

fn legacy_single_probe_scene_trace_support(
    state: &HybridGiRuntimeState,
    probe_id: u32,
    region_resolutions: &mut usize,
) -> f32 {
    let scheduled_trace_regions = state.resolve_scheduled_scene_trace_regions();
    *region_resolutions += 1;
    state.single_probe_scene_trace_support(probe_id, &scheduled_trace_regions)
}

fn legacy_current_lineage_trace_support_score(
    state: &HybridGiRuntimeState,
    probe_id: u32,
    region_resolutions: &mut usize,
) -> f32 {
    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        total_support +=
            legacy_single_probe_scene_trace_support(state, current_probe_id, region_resolutions)
                * lineage_weight;
        let Some(parent_probe_id) = state.probe_parent_probes().get(&current_probe_id).copied()
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        lineage_weight *= ANCESTOR_TRACE_SUPPORT_FALLOFF;
        current_probe_id = parent_probe_id;
    }

    let mut descendant_support = 0.0_f32;
    for (candidate_probe_id, depth) in state.probe_descendant_ids_with_depth(probe_id) {
        descendant_support = descendant_support.max(
            legacy_single_probe_scene_trace_support(state, candidate_probe_id, region_resolutions)
                * DESCENDANT_TRACE_SUPPORT_FALLOFF.powi((depth - 1) as i32),
        );
    }
    total_support + descendant_support
}

fn legacy_refresh_recent_lineage_trace_support(state: &mut HybridGiRuntimeState) -> usize {
    let mut region_resolutions = 0;
    let probe_ids = state.probe_scene_data().keys().copied().collect::<Vec<_>>();
    for probe_id in probe_ids {
        let current_q8 = quantize_support_q8(legacy_current_lineage_trace_support_score(
            state,
            probe_id,
            &mut region_resolutions,
        ));
        let decayed_recent_q8 = state
            .recent_lineage_trace_support_q8()
            .get(&probe_id)
            .copied()
            .map(decay_support_q8)
            .unwrap_or_default();
        let refreshed_q8 = current_q8.max(decayed_recent_q8);
        if refreshed_q8 == 0 {
            state
                .recent_lineage_trace_support_q8_mut()
                .remove(&probe_id);
        } else {
            state
                .recent_lineage_trace_support_q8_mut()
                .insert(probe_id, refreshed_q8);
        }

        let current_request_q8 =
            quantize_support_q8(state.current_requested_lineage_support_score(probe_id));
        let decayed_recent_request_q8 = state
            .recent_requested_lineage_support_q8()
            .get(&probe_id)
            .copied()
            .map(decay_request_support_q8)
            .unwrap_or_default();
        let refreshed_request_q8 = current_request_q8.max(decayed_recent_request_q8);
        if refreshed_request_q8 == 0 {
            state
                .recent_requested_lineage_support_q8_mut()
                .remove(&probe_id);
        } else {
            state
                .recent_requested_lineage_support_q8_mut()
                .insert(probe_id, refreshed_request_q8);
        }
    }
    region_resolutions
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut samples = samples.to_vec();
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100).max(1);
    samples[rank - 1]
}

#[test]
fn runtime98_lineage_trace_support_batch_preserves_legacy_scores() {
    let mut legacy = fixture();
    let mut optimized = legacy.clone();

    let legacy_region_resolutions = legacy_refresh_recent_lineage_trace_support(&mut legacy);
    optimized.refresh_recent_lineage_trace_support();

    assert_eq!(
        legacy_region_resolutions,
        LEGACY_REGION_RESOLUTIONS_PER_REFRESH
    );
    assert_eq!(
        optimized.recent_lineage_trace_support_q8(),
        legacy.recent_lineage_trace_support_q8()
    );
    assert_eq!(
        optimized.recent_requested_lineage_support_q8(),
        legacy.recent_requested_lineage_support_q8()
    );
}

#[test]
#[ignore = "release performance evidence"]
fn runtime98_lineage_trace_support_release_benchmark_evidence() {
    let mut legacy_state = fixture();
    let mut optimized_state = legacy_state.clone();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let run_legacy = |state: &mut HybridGiRuntimeState| {
            let started = Instant::now();
            let resolutions = legacy_refresh_recent_lineage_trace_support(black_box(&mut *state));
            let elapsed = started.elapsed();
            assert_eq!(resolutions, LEGACY_REGION_RESOLUTIONS_PER_REFRESH);
            black_box(state.recent_lineage_trace_support_q8().len());
            elapsed
        };
        let run_optimized = |state: &mut HybridGiRuntimeState| {
            let started = Instant::now();
            black_box(&mut *state).refresh_recent_lineage_trace_support();
            let elapsed = started.elapsed();
            black_box(state.recent_lineage_trace_support_q8().len());
            elapsed
        };

        if pair % 2 == 0 {
            legacy_samples.push(run_legacy(&mut legacy_state));
            optimized_samples.push(run_optimized(&mut optimized_state));
        } else {
            optimized_samples.push(run_optimized(&mut optimized_state));
            legacy_samples.push(run_legacy(&mut legacy_state));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    assert!(optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60);
    let duration_csv = |samples: &[Duration]| {
        samples
            .iter()
            .map(|sample| sample.as_nanos().to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    println!(
        "RUNTIME98_LINEAGE_TRACE_SUPPORT_PERF probes={} regions={} sample_pairs={} sample_order=alternating_legacy_first_even percentile_method=nearest_rank threshold_percent=40 legacy_ns={} optimized_ns={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_region_resolutions=12286 optimized_region_resolutions=1",
        PROBE_COUNT,
        TRACE_REGION_COUNT,
        SAMPLE_PAIRS,
        duration_csv(&legacy_samples),
        duration_csv(&optimized_samples),
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
}
