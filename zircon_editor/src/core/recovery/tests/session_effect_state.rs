use std::hint::black_box;
use std::time::Instant;

use super::{
    session_effect_states, ProjectSessionEffect, ProjectSessionEffectDisposition,
    ProjectSessionEffectRecoveryEntry,
};

const EFFECT_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor884_session_effect_state_single_buffer_preserves_exact_text() {
    assert_eq!(session_effect_states(&[]), "");

    let effects = [
        ProjectSessionEffectRecoveryEntry::new(
            ProjectSessionEffect::Runtime,
            ProjectSessionEffectDisposition::Prepared,
        ),
        ProjectSessionEffectRecoveryEntry::new(
            ProjectSessionEffect::Documents,
            ProjectSessionEffectDisposition::Committed,
        ),
        ProjectSessionEffectRecoveryEntry::new(
            ProjectSessionEffect::FocusBinding,
            ProjectSessionEffectDisposition::RolledBack,
        ),
        ProjectSessionEffectRecoveryEntry::new(
            ProjectSessionEffect::WorkspaceProjection,
            ProjectSessionEffectDisposition::RecoveryRequired,
        ),
    ];
    assert_eq!(
        session_effect_states(&effects),
        "runtime=Prepared, documents=Committed, focus_binding=RolledBack, workspace_projection=RecoveryRequired"
    );
    assert_eq!(
        session_effect_states(&effects),
        legacy_session_effect_states(&effects)
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor884_session_effect_state_single_buffer_benchmark() {
    let effects = dense_effects();
    assert_eq!(
        session_effect_states(&effects),
        legacy_session_effect_states(&effects)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_session_effect_states(&effects)));
            optimized.push(measure(|| session_effect_states(&effects)));
        } else {
            optimized.push(measure(|| session_effect_states(&effects)));
            legacy.push(measure(|| legacy_session_effect_states(&effects)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR884_SESSION_EFFECT_STATE_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} effects={EFFECT_COUNT} legacy_temporary_strings={EFFECT_COUNT} legacy_temporary_vector_slots={EFFECT_COUNT} optimized_temporary_strings=0 optimized_temporary_vector_slots=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "single-buffer formatting P95 {optimized_p95}ns must stay within 10% of temporary-vector formatting P95 {legacy_p95}ns"
    );
}

fn dense_effects() -> Vec<ProjectSessionEffectRecoveryEntry> {
    let effects = [
        ProjectSessionEffect::Runtime,
        ProjectSessionEffect::Documents,
        ProjectSessionEffect::FocusBinding,
        ProjectSessionEffect::WorkspaceProjection,
    ];
    let dispositions = [
        ProjectSessionEffectDisposition::Prepared,
        ProjectSessionEffectDisposition::Committed,
        ProjectSessionEffectDisposition::RolledBack,
        ProjectSessionEffectDisposition::RecoveryRequired,
    ];
    (0..EFFECT_COUNT)
        .map(|index| {
            ProjectSessionEffectRecoveryEntry::new(
                effects[index % effects.len()],
                dispositions[index % dispositions.len()],
            )
        })
        .collect()
}

fn legacy_session_effect_states(effects: &[ProjectSessionEffectRecoveryEntry]) -> String {
    effects
        .iter()
        .map(|entry| format!("{}={:?}", entry.effect().as_str(), entry.disposition()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
