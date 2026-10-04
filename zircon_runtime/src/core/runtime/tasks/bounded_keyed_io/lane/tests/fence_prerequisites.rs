use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;
use crate::core::runtime::tasks::bounded_keyed_io::BoundedKeyedIoTicket;

const BENCHMARK_PREREQUISITE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLES: usize = 11;
const BENCHMARK_ITERATIONS: usize = 64;

#[test]
fn reused_visiting_set_preserves_fence_results() {
    let success = terminal_prerequisite(1, "asset:a", 1, BoundedKeyedIoTerminal::Succeeded);
    let failed = terminal_prerequisite(
        2,
        "asset:b",
        1,
        BoundedKeyedIoTerminal::Failed(BoundedKeyedIoFailure::new("write_failed")),
    );
    let superseded = terminal_prerequisite(
        3,
        "asset:c",
        1,
        BoundedKeyedIoTerminal::Superseded { successor: 2 },
    );
    let successor = terminal_prerequisite(4, "asset:c", 2, BoundedKeyedIoTerminal::Succeeded);

    for prerequisites in [
        vec![success.clone()],
        vec![success, failed],
        vec![superseded, successor],
    ] {
        assert_eq!(
            fence_prerequisite_failure(&prerequisites),
            retired_fence_prerequisite_failure(&prerequisites)
        );
    }
}

#[test]
fn fence_failure_reuses_one_visiting_set_across_roots() {
    let source = include_str!("../fence_prerequisites.rs");
    let implementation = source
        .split_once("pub(super) fn fence_prerequisite_failure")
        .expect("fence prerequisite failure function")
        .1
        .split_once("\n}\n\nfn fence_prerequisite")
        .expect("fence prerequisite failure function end")
        .0;
    let per_root_set = ["&mut HashSet", "::new()"].concat();

    assert!(!implementation.contains(&per_root_set));
    assert_eq!(implementation.matches("HashSet::new()").count(), 1);
    assert!(implementation.contains("visiting.clear()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn reused_fence_visiting_set_release_benchmark() {
    let prerequisites = (0..BENCHMARK_PREREQUISITE_COUNT)
        .map(|index| {
            terminal_prerequisite(
                index as u64 + 1,
                &format!("asset:{index:04}"),
                1,
                BoundedKeyedIoTerminal::Succeeded,
            )
        })
        .collect::<Vec<_>>();
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_failure_scan(|| {
                retired_fence_prerequisite_failure(&prerequisites)
            }));
            optimized_samples.push(measure_failure_scan(|| {
                fence_prerequisite_failure(&prerequisites)
            }));
        } else {
            optimized_samples.push(measure_failure_scan(|| {
                fence_prerequisite_failure(&prerequisites)
            }));
            retired_samples.push(measure_failure_scan(|| {
                retired_fence_prerequisite_failure(&prerequisites)
            }));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME59_REUSED_FENCE_VISITING_SET_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
prerequisites={BENCHMARK_PREREQUISITE_COUNT} \
retired_visiting_set_allocations_per_scan=4096 optimized_visiting_set_allocations_per_scan=1 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(75),
        "reused visiting set must reduce fence failure scan P95 by at least 25%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

fn terminal_prerequisite(
    id: u64,
    key: &str,
    generation: u64,
    terminal: BoundedKeyedIoTerminal,
) -> FencePrerequisite {
    let ticket = BoundedKeyedIoTicket::pending(id, generation, false);
    assert!(ticket.mark_terminal(terminal));
    FencePrerequisite {
        key: Some(Arc::<str>::from(key).into()),
        generation,
        ticket,
    }
}

fn measure_failure_scan(mut scan: impl FnMut() -> Option<BoundedKeyedIoFailure>) -> Duration {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        assert_eq!(black_box(scan()), None);
    }
    started.elapsed()
}

fn retired_fence_prerequisite_failure(
    prerequisites: &[FencePrerequisite],
) -> Option<BoundedKeyedIoFailure> {
    prerequisites.iter().find_map(|prerequisite| {
        prerequisite_result(prerequisite, prerequisites, &mut HashSet::new()).err()
    })
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
