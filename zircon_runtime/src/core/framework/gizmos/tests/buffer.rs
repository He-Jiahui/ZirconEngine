use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use super::*;

fn legacy_linestrip(buffer: &mut GizmoBuffer, points: impl IntoIterator<Item = Vec3>, color: Vec4) {
    buffer.push_command(GizmoCommand::LineStrip {
        points: points.into_iter().collect(),
        color,
    });
}

#[test]
fn optimization_batch_eq_disabled_linestrip_skips_temporary_point_storage() {
    let mut buffer = GizmoBuffer::new();
    buffer.config_mut().enabled = false;
    let visited = Cell::new(0_usize);
    let points = (0..1_024).map(|index| {
        visited.set(visited.get() + 1);
        Vec3::splat(index as Real)
    });

    buffer.linestrip(points, Vec4::ONE);

    assert_eq!(visited.get(), 1_024);
    assert!(buffer.commands().is_empty());

    let source = include_str!("../buffer.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("gizmo buffer production implementation");
    let linestrip = production
        .split("pub fn linestrip(")
        .nth(1)
        .expect("line strip implementation")
        .split("pub fn rect(")
        .next()
        .expect("line strip function body");
    let disabled_check = linestrip
        .find("if !self.config.enabled")
        .expect("disabled line strip check");
    let collect = linestrip
        .find("collect()")
        .expect("enabled point collection");
    assert!(disabled_check < collect);
}

#[test]
#[ignore = "release-only disabled line strip allocation benchmark"]
fn optimization_batch_eq_disabled_linestrip_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const STRIPS_PER_SAMPLE: usize = 256;
    const POINTS_PER_STRIP: usize = 1_024;

    fn measure_legacy(seed: &[Vec3]) -> u128 {
        let mut buffer = GizmoBuffer::new();
        buffer.config_mut().enabled = false;
        let started = Instant::now();
        for _ in 0..STRIPS_PER_SAMPLE {
            legacy_linestrip(
                &mut buffer,
                seed.iter().copied().inspect(|point| {
                    black_box(point);
                }),
                Vec4::ONE,
            );
        }
        black_box(buffer.commands().len());
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(seed: &[Vec3]) -> u128 {
        let mut buffer = GizmoBuffer::new();
        buffer.config_mut().enabled = false;
        let started = Instant::now();
        for _ in 0..STRIPS_PER_SAMPLE {
            buffer.linestrip(
                seed.iter().copied().inspect(|point| {
                    black_box(point);
                }),
                Vec4::ONE,
            );
        }
        black_box(buffer.commands().len());
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

    let seed = (0..POINTS_PER_STRIP)
        .map(|index| Vec3::new(index as Real, 1.0, -1.0))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_legacy(&seed));
        black_box(measure_optimized(&seed));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&seed));
            optimized_samples.push(measure_optimized(&seed));
        } else {
            optimized_samples.push(measure_optimized(&seed));
            legacy_samples.push(measure_legacy(&seed));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME449_DISABLED_LINESTRIP_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             strips_per_sample={STRIPS_PER_SAMPLE} points_per_strip={POINTS_PER_STRIP} \
             pair_order=alternating_legacy_even legacy_point_allocations_per_strip=1 \
             optimized_point_allocations_per_strip=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
        "disabled line strips must reduce P95 by at least 25%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
