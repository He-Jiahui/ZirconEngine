use std::collections::{HashSet, VecDeque};
use std::hint::black_box;
use std::time::Instant;

const COMPONENT_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 31;

#[test]
fn optimization_batch_iw_editor633_reserves_widget_dependency_closure() {
    let source = include_str!("../../promote_widget.rs");
    let closure_body = source
        .split("fn collect_component_dependency_names")
        .nth(1)
        .expect("component dependency closure remains present")
        .split("fn collect_local_component_references")
        .next()
        .expect("component dependency closure remains bounded");

    assert!(closure_body.contains("let dependency_capacity = document.components.len();"));
    assert!(closure_body.contains("HashSet::with_capacity(dependency_capacity)"));
    assert!(closure_body.contains("VecDeque::with_capacity(dependency_capacity)"));
    assert!(!closure_body.contains("HashSet::new()"));
    assert!(!closure_body.contains("VecDeque::from("));
}

#[test]
#[ignore = "release helper microbenchmark; real promotion caller evidence is required"]
fn optimization_batch_iw_editor633_preallocated_widget_dependency_benchmark() {
    for _ in 0..4 {
        black_box(measure_dependency_closure(false));
        black_box(measure_dependency_closure(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_dependency_closure(false));
            preallocated_samples.push(measure_dependency_closure(true));
        } else {
            preallocated_samples.push(measure_dependency_closure(true));
            unreserved_samples.push(measure_dependency_closure(false));
        }
    }

    let unreserved_p50 = percentile(&unreserved_samples, 50);
    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let unreserved_p99 = percentile(&unreserved_samples, 99);
    let preallocated_p50 = percentile(&preallocated_samples, 50);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let preallocated_p99 = percentile(&preallocated_samples, 99);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR633_PREALLOCATED_WIDGET_DEPENDENCY_CLOSURE_BENCH_V2 \
         sample_pairs={SAMPLE_PAIRS} component_count={COMPONENT_COUNT} \
         benchmark_scope=microbenchmark real_caller_required=true \
         percentile_method=nearest_rank unreserved_p50_ns={unreserved_p50} \
         unreserved_p95_ns={unreserved_p95} unreserved_p99_ns={unreserved_p99} \
         preallocated_p50_ns={preallocated_p50} preallocated_p95_ns={preallocated_p95} \
         preallocated_p99_ns={preallocated_p99} unreserved_ns={} preallocated_ns={} \
         improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_dependency_closure(preallocated: bool) -> u128 {
    let mut visited = if preallocated {
        HashSet::with_capacity(COMPONENT_COUNT)
    } else {
        HashSet::new()
    };
    let mut pending = if preallocated {
        VecDeque::with_capacity(COMPONENT_COUNT)
    } else {
        VecDeque::new()
    };
    pending.push_back(0usize);
    let started = Instant::now();
    while let Some(component) = pending.pop_front() {
        if !visited.insert(component) {
            continue;
        }
        if component + 1 < COMPONENT_COUNT {
            pending.push_back(component + 1);
        }
    }
    black_box((visited, pending));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
