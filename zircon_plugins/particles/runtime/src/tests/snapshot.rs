use std::{hint::black_box, sync::Arc, time::Instant};

use zircon_runtime::core::framework::render::RenderParticleSpriteSnapshot;

use crate::{
    ParticleAnimationEvent, ParticleRuntimeDiagnostic, ParticleRuntimeSnapshot,
    ParticleSimulationBackend, ParticleSystemComponent, ParticlesManager,
};

use super::support::spawn_rate_asset;

const SNAPSHOT_BENCH_PAIRS: usize = 21;
const SNAPSHOT_BENCH_ITERATIONS: usize = 128;
const SNAPSHOT_BENCH_SPRITES: usize = 4_096;
const SNAPSHOT_BENCH_DIAGNOSTICS: usize = 256;

#[test]
fn unchanged_snapshots_share_payloads_until_simulation_changes() {
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(
            42,
            spawn_rate_asset(8.0, 32).with_backend(ParticleSimulationBackend::Gpu),
        ))
        .unwrap();
    manager.tick(0.25).unwrap();

    let first = manager.snapshot();
    let second = manager.snapshot();
    assert!(Arc::ptr_eq(&first.sprites, &second.sprites));
    assert!(Arc::ptr_eq(&first.diagnostics, &second.diagnostics));

    manager.tick(0.25).unwrap();
    let changed = manager.snapshot();
    assert!(!Arc::ptr_eq(&first.sprites, &changed.sprites));
    assert!(Arc::ptr_eq(&first.diagnostics, &changed.diagnostics));
}

#[test]
fn diagnostics_are_bounded_sequenced_pageable_and_acknowledgeable() {
    let manager = ParticlesManager::default();
    for entity in 0..300 {
        manager
            .apply_animation_event(ParticleAnimationEvent::spawn_once(entity))
            .unwrap();
    }

    let snapshot = manager.snapshot();
    assert_eq!(snapshot.diagnostics.len(), 256);
    assert_eq!(snapshot.diagnostic_sequence, 300);
    assert_eq!(snapshot.dropped_diagnostics, 44);

    let first_page = manager.diagnostics_page(0, usize::MAX);
    assert!(first_page.stale_cursor);
    assert_eq!(first_page.oldest_available_sequence, 45);
    assert_eq!(first_page.entries.len(), 64);
    assert_eq!(first_page.entries[0].sequence, 45);
    assert_eq!(first_page.next_sequence, 108);
    assert_eq!(first_page.dropped_total, 44);

    let diagnostics_before_ack = Arc::clone(&snapshot.diagnostics);
    assert_eq!(
        manager.acknowledge_diagnostics(first_page.next_sequence),
        64
    );
    let next_page = manager.diagnostics_page(first_page.next_sequence, usize::MAX);
    assert!(!next_page.stale_cursor);
    assert_eq!(next_page.oldest_available_sequence, 109);
    assert_eq!(next_page.entries[0].sequence, 109);
    let acknowledged_snapshot = manager.snapshot();
    assert_eq!(acknowledged_snapshot.diagnostics.len(), 192);
    assert!(!Arc::ptr_eq(
        &diagnostics_before_ack,
        &acknowledged_snapshot.diagnostics
    ));
}

#[test]
#[ignore = "release performance gate"]
fn particle_snapshot_shared_clone_release_benchmark() {
    let snapshot = benchmark_snapshot();
    let mut legacy_samples = Vec::with_capacity(SNAPSHOT_BENCH_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SNAPSHOT_BENCH_PAIRS);
    for pair_index in 0..SNAPSHOT_BENCH_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_snapshot_clones(&snapshot, true));
            optimized_samples.push(measure_snapshot_clones(&snapshot, false));
        } else {
            optimized_samples.push(measure_snapshot_clones(&snapshot, false));
            legacy_samples.push(measure_snapshot_clones(&snapshot, true));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let legacy_payload_clones =
        SNAPSHOT_BENCH_ITERATIONS * (SNAPSHOT_BENCH_SPRITES + SNAPSHOT_BENCH_DIAGNOSTICS);

    println!(
        "PARTICLE_SNAPSHOT_SHARE_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank iterations={} sprites={} diagnostics={} legacy_payload_clones={} optimized_payload_clones=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        SNAPSHOT_BENCH_PAIRS,
        SNAPSHOT_BENCH_ITERATIONS,
        SNAPSHOT_BENCH_SPRITES,
        SNAPSHOT_BENCH_DIAGNOSTICS,
        legacy_payload_clones,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(4) <= legacy_p95,
        "shared snapshot P95 must be at most 25% of legacy: legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn benchmark_snapshot() -> ParticleRuntimeSnapshot {
    ParticleRuntimeSnapshot {
        sprites: Arc::from(
            vec![RenderParticleSpriteSnapshot::default(); SNAPSHOT_BENCH_SPRITES]
                .into_boxed_slice(),
        ),
        diagnostics: Arc::from(
            vec![
                ParticleRuntimeDiagnostic::warning(None, "bounded particle diagnostic payload");
                SNAPSHOT_BENCH_DIAGNOSTICS
            ]
            .into_boxed_slice(),
        ),
        diagnostic_sequence: SNAPSHOT_BENCH_DIAGNOSTICS as u64,
        ..ParticleRuntimeSnapshot::default()
    }
}

fn measure_snapshot_clones(snapshot: &ParticleRuntimeSnapshot, legacy: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..SNAPSHOT_BENCH_ITERATIONS {
        if legacy {
            black_box(legacy_clone_snapshot(snapshot));
        } else {
            black_box(snapshot.clone());
        }
    }
    started.elapsed().as_nanos()
}

fn legacy_clone_snapshot(snapshot: &ParticleRuntimeSnapshot) -> ParticleRuntimeSnapshot {
    ParticleRuntimeSnapshot {
        emitters: snapshot.emitters.clone(),
        sprites: Arc::from(snapshot.sprites.as_ref().to_vec()),
        diagnostics: Arc::from(snapshot.diagnostics.as_ref().to_vec()),
        diagnostic_sequence: snapshot.diagnostic_sequence,
        dropped_diagnostics: snapshot.dropped_diagnostics,
        last_gpu_feedback: snapshot.last_gpu_feedback.clone(),
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
