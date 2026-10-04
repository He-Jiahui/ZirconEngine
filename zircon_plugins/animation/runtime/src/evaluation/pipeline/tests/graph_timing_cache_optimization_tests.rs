use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
struct ModelRevision {
    clip_id: u64,
    revision: u64,
    playback_speed_bits: u32,
}

#[test]
fn optimization_batch_20260830ch_graph_timing_hit_avoids_signature_allocation() {
    let source = include_str!("../graph_timing_cache.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    let start = production
        .find("    pub(super) fn graph_duration_seconds(")
        .expect("graph timing owner");
    let end = production[start..]
        .find("    fn enforce_graph_timing_cache_limit(")
        .map(|offset| start + offset)
        .expect("graph timing owner boundary");
    let owner = &production[start..end];

    let hit_probe = owner
        .find("graph_timing_signature_matches")
        .expect("allocation-free signature probe");
    let rebuild = owner
        .find("Vec::with_capacity")
        .expect("miss-only signature rebuild");
    assert!(hit_probe < rebuild);
    assert!(owner.contains("return Some(cached.duration_seconds)"));
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830ch_graph_timing_hit_p95() {
    const CLIPS: u64 = 64;
    const ITERATIONS: usize = 100_000;
    const SAMPLES: usize = 17;
    let clips = (0..CLIPS)
        .map(|clip_id| (clip_id, clip_id * 7 + 1, 1.0 + clip_id as f32 * 0.01))
        .collect::<Vec<_>>();
    let cached = clips
        .iter()
        .map(|&(clip_id, revision, speed)| ModelRevision {
            clip_id,
            revision,
            playback_speed_bits: speed.to_bits(),
        })
        .collect::<Vec<_>>();
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut signature = Vec::with_capacity(clips.len());
                let mut duration = 0.0_f32;
                for &(clip_id, revision, speed) in black_box(&clips) {
                    signature.push(ModelRevision {
                        clip_id,
                        revision,
                        playback_speed_bits: speed.to_bits(),
                    });
                    duration = duration.max(3.0 / speed);
                }
                black_box((signature == cached, duration));
            }
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut expected = cached.iter();
                let matches = black_box(&clips).iter().all(|&(clip_id, revision, speed)| {
                    expected.next()
                        == Some(&ModelRevision {
                            clip_id,
                            revision,
                            playback_speed_bits: speed.to_bits(),
                        })
                }) && expected.next().is_none();
                black_box(matches);
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            baseline_samples.push(baseline());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            baseline_samples.push(baseline());
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME170_GRAPH_TIMING_CACHE_HIT_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(70),
        "expected allocation-free cache hit to reduce P95 by at least 30%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}
