use super::*;
use crate::ui::event_ui::{UiNodeId, UiNodePath};

fn projection(node_id: u64, render_command_count: u64) -> UiEcsNodeProjection {
    UiEcsNodeProjection {
        node_id: UiNodeId::new(node_id),
        node_path: UiNodePath::new(format!("root/{node_id}")),
        render_command_count,
        ..UiEcsNodeProjection::default()
    }
}

#[test]
fn runtime_interface03_batch4_ordered_projection_diff_matches_map_fallback_order_and_reasons() {
    let previous = vec![projection(2, 0), projection(4, 0), projection(6, 0)];
    let current = vec![
        projection(1, 0),
        projection(2, 1),
        projection(6, 0),
        projection(8, 0),
    ];

    assert!(nodes_are_strictly_ordered(&previous));
    assert!(nodes_are_strictly_ordered(&current));
    assert_eq!(
        ordered_projection_changes(&previous, &current),
        mapped_projection_changes(&previous, &current),
    );
}

#[test]
fn runtime_interface03_batch4_unordered_or_duplicate_projection_diff_uses_map_semantics() {
    let previous = vec![projection(2, 0), projection(1, 0), projection(1, 1)];
    let current = vec![projection(2, 1), projection(3, 0)];

    assert!(!nodes_are_strictly_ordered(&previous));
    assert_eq!(
        projection_changes(&previous, &current),
        mapped_projection_changes(&previous, &current),
    );
}

#[test]
#[ignore = "release-only linear sorted ECS diff benchmark"]
fn runtime_interface03_batch4_linear_sorted_ecs_diff_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const NODE_COUNT: usize = 4_096;
    const EDGE_CHURN: usize = 128;
    const SAMPLE_COUNT: usize = 11;
    let previous = (0..NODE_COUNT)
        .map(|node_id| projection(node_id as u64, 0))
        .collect::<Vec<_>>();
    let current = (EDGE_CHURN..(NODE_COUNT + EDGE_CHURN))
        .map(|node_id| projection(node_id as u64, u64::from(node_id % 16 == 0)))
        .collect::<Vec<_>>();
    let mut mapped_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_mapped = || {
            let started = Instant::now();
            black_box(mapped_projection_changes(
                black_box(&previous),
                black_box(&current),
            ));
            started.elapsed().as_nanos()
        };
        let measure_linear = || {
            let started = Instant::now();
            black_box(ordered_projection_changes(
                black_box(&previous),
                black_box(&current),
            ));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            mapped_samples.push(measure_mapped());
            linear_samples.push(measure_linear());
        } else {
            linear_samples.push(measure_linear());
            mapped_samples.push(measure_mapped());
        }
    }

    mapped_samples.sort_unstable();
    linear_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_LINEAR_SORTED_ECS_DIFF_BENCH_V1 nodes={NODE_COUNT} edge_churn={EDGE_CHURN} samples={SAMPLE_COUNT} mapped_p50_ns={} linear_p50_ns={} mapped_p95_ns={} linear_p95_ns={}",
        mapped_samples[p50],
        linear_samples[p50],
        mapped_samples[p95],
        linear_samples[p95],
    );
    assert!(
        linear_samples[p95].saturating_mul(5) <= mapped_samples[p95].saturating_mul(4),
        "linear sorted ECS diff must improve P95 by at least 20%: mapped={}ns linear={}ns",
        mapped_samples[p95],
        linear_samples[p95],
    );
}
