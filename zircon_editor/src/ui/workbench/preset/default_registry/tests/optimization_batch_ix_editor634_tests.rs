use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const VIEW_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ix_editor634_preserves_unique_default_view_ids() {
    let instances = EditorUiDesignStack::material_fyrox_jetbrains_unreal().default_view_instances();
    let unique = instances
        .iter()
        .map(|instance| instance.instance_id.clone())
        .collect::<HashSet<_>>();

    assert_eq!(unique.len(), instances.len());
}

#[test]
fn optimization_batch_ix_editor634_reserves_default_view_instances() {
    let source = include_str!("../../default_registry.rs");
    let projection_body = source
        .split("pub fn default_view_instances")
        .nth(1)
        .expect("default view projection remains present")
        .split("pub fn default_window_registry")
        .next()
        .expect("default view projection remains bounded");

    assert!(projection_body.contains("let instance_capacity ="));
    assert!(projection_body.contains("window.primary_views.len()"));
    assert!(projection_body.contains("window.drawer_views.len()"));
    assert!(projection_body.contains("HashSet::with_capacity(instance_capacity)"));
    assert!(projection_body.contains("Vec::with_capacity(instance_capacity)"));
    assert!(!projection_body.contains("let mut seen = HashSet::new();"));
    assert!(!projection_body.contains("let mut instances = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ix_editor634_preallocated_default_view_benchmark() {
    let ids = (0..VIEW_COUNT)
        .map(|index| format!("editor.synthetic.view.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_projection(&ids, false));
        black_box(measure_projection(&ids, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_projection(&ids, false));
            preallocated_samples.push(measure_projection(&ids, true));
        } else {
            preallocated_samples.push(measure_projection(&ids, true));
            unreserved_samples.push(measure_projection(&ids, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR634_PREALLOCATED_DEFAULT_VIEW_INSTANCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} view_count={VIEW_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_projection(ids: &[String], preallocated: bool) -> u128 {
    let mut seen = if preallocated {
        HashSet::with_capacity(ids.len())
    } else {
        HashSet::new()
    };
    let mut output = if preallocated {
        Vec::with_capacity(ids.len())
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for id in ids {
        if seen.insert(id.as_str()) {
            output.push(id.as_str());
        }
    }
    black_box((seen, output));
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
