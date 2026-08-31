use std::hint::black_box;
use std::time::Instant;

use super::*;

const PLAYER_COUNT: usize = 4 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826ho_runtime261_preserves_sanitized_snapshot_semantics() {
    let world = WorldHandle::new(17);
    let mut player = AnimationPlayerRuntimeStatus::new(world, 23, AnimationPlayerKind::Clip);
    player.time_seconds = f32::NAN;
    player.playback_speed = -2.0;
    player.weight = 2.0;
    player.diagnostics.push("source diagnostic".to_string());
    let status = AnimationRuntimeStatus::new(world)
        .with_player(player)
        .with_diagnostic("runtime diagnostic");

    let snapshot = status.sanitized_snapshot();

    assert_eq!(snapshot.players[0].time_seconds, 0.0);
    assert_eq!(snapshot.players[0].playback_speed, 0.0);
    assert_eq!(snapshot.players[0].weight, 1.0);
    assert_eq!(
        snapshot.players[0].diagnostics,
        vec!["source diagnostic".to_string()]
    );
    assert_eq!(snapshot.diagnostics, vec!["runtime diagnostic".to_string()]);
    assert!(status.players[0].time_seconds.is_nan());
    assert_eq!(status.players[0].playback_speed, -2.0);
}

#[test]
fn optimization_batch_20260826ho_runtime261_builds_snapshot_without_whole_status_clone() {
    let source = include_str!("../runtime_status.rs");
    let start = source
        .rfind("pub fn sanitized_snapshot(&self) -> Self")
        .expect("runtime snapshot function");
    let body = &source[start..];

    assert!(body.contains("players: self"));
    assert!(body.contains("rigs: self.rigs.clone()"));
    assert!(body.contains("last_tick: self.last_tick.clone()"));
    assert!(body.contains("diagnostics: self.diagnostics.clone()"));
    assert!(!body.contains("let mut snapshot = self.clone()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826ho_runtime261_single_pass_animation_snapshot_release_benchmark() {
    let source = benchmark_status();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_sanitized_snapshot(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(black_box(&source).sanitized_snapshot());
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME261_SINGLE_PASS_ANIMATION_SNAPSHOT_BENCH_V1 \
         player_count={PLAYER_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_status() -> AnimationRuntimeStatus {
    let world = WorldHandle::new(29);
    let mut status = AnimationRuntimeStatus::new(world);
    status.players.reserve(PLAYER_COUNT);
    for index in 0..PLAYER_COUNT {
        let mut player = AnimationPlayerRuntimeStatus::new(
            world,
            index as EntityId,
            AnimationPlayerKind::StateMachine,
        );
        player.active_state = Some(format!("state-{index:08}-{}", "x".repeat(64)));
        player.diagnostics = vec![format!("diagnostic-{index:08}-{}", "y".repeat(64))];
        player.time_seconds = f32::NAN;
        player.playback_speed = -1.0;
        player.weight = 2.0;
        status.players.push(player);
    }
    status
}

fn legacy_sanitized_snapshot(source: &AnimationRuntimeStatus) -> AnimationRuntimeStatus {
    let mut snapshot = source.clone();
    snapshot.players = source
        .players
        .iter()
        .map(AnimationPlayerRuntimeStatus::sanitized_snapshot)
        .collect();
    snapshot
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
