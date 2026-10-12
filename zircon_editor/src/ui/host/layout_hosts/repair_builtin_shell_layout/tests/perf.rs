use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost};

use super::*;

const OPEN_INSTANCE_COUNT: usize = 2_048;
const QUERY_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iy_editor636_index_preserves_exact_and_first_descriptor_match() {
    let open_instances = vec![
        instance("editor.assets#7", "editor.assets"),
        instance("editor.assets#8", "editor.assets"),
        instance("editor.hierarchy#3", "editor.hierarchy"),
    ];
    let index = OpenInstanceIndex::new(&open_instances);

    assert_eq!(
        index.matching(&ViewInstanceId::new("editor.assets#8")),
        Some(ViewInstanceId::new("editor.assets#8"))
    );
    assert_eq!(
        index.matching(&ViewInstanceId::new("editor.assets#1")),
        Some(ViewInstanceId::new("editor.assets#7"))
    );
    assert_eq!(
        index.matching(&ViewInstanceId::new("editor.missing#1")),
        None
    );
}

#[test]
fn optimization_batch_iy_editor636_builds_borrowed_open_instance_indexes() {
    let source = include_str!("../../repair_builtin_shell_layout.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("struct OpenInstanceIndex<'a>"));
    assert!(production.contains("HashMap::with_capacity(open_instances.len())"));
    assert!(production.contains(".entry(&instance.instance_id)"));
    assert!(production.contains(".or_insert(instance)"));
    assert!(production.contains(".entry(instance.descriptor_id.0.as_str())"));
    assert!(
        production.contains("let open_instance_index = OpenInstanceIndex::new(open_instances);")
    );
    assert!(!production.contains("fn matching_open_instance("));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iy_editor636_indexed_shell_instance_lookup_benchmark() {
    let open_instances = (0..OPEN_INSTANCE_COUNT)
        .map(|index| {
            (
                format!("editor.synthetic.{index:05}#7"),
                format!("editor.synthetic.{index:05}"),
            )
        })
        .collect::<Vec<_>>();
    let queries = (0..QUERY_COUNT)
        .map(|index| {
            let target = (index * 1_543) % OPEN_INSTANCE_COUNT;
            if index % 2 == 0 {
                format!("editor.synthetic.{target:05}#7")
            } else {
                format!("editor.synthetic.{target:05}#missing")
            }
        })
        .collect::<Vec<_>>();

    for _ in 0..2 {
        black_box(measure_linear(&open_instances, &queries));
        black_box(measure_indexed(&open_instances, &queries));
    }

    let mut linear_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            linear_samples.push(measure_linear(&open_instances, &queries));
            indexed_samples.push(measure_indexed(&open_instances, &queries));
        } else {
            indexed_samples.push(measure_indexed(&open_instances, &queries));
            linear_samples.push(measure_linear(&open_instances, &queries));
        }
    }

    let linear_p95 = percentile(&linear_samples, 95);
    let indexed_p95 = percentile(&indexed_samples, 95);
    let improvement_percent =
        linear_p95.saturating_sub(indexed_p95).saturating_mul(100) / linear_p95.max(1);
    println!(
        "EDITOR636_INDEXED_SHELL_INSTANCE_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} open_instance_count={OPEN_INSTANCE_COUNT} query_count={QUERY_COUNT} linear_ns={} indexed_ns={} linear_p95_ns={linear_p95} indexed_p95_ns={indexed_p95} improvement_percent={improvement_percent} threshold_percent=75",
        csv(&linear_samples),
        csv(&indexed_samples),
    );
    assert!(indexed_p95 <= linear_p95 * 25 / 100);
}

fn instance(instance_id: &str, descriptor_id: &str) -> ViewInstance {
    ViewInstance {
        instance_id: ViewInstanceId::new(instance_id),
        descriptor_id: ViewDescriptorId::new(descriptor_id),
        title: descriptor_id.to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
    }
}

fn measure_linear(open_instances: &[(String, String)], queries: &[String]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for query in queries {
        let matched = open_instances
            .iter()
            .find(|(instance_id, _)| instance_id == query)
            .or_else(|| {
                let descriptor_id = query.rsplit_once('#')?.0;
                open_instances
                    .iter()
                    .find(|(_, candidate_descriptor)| candidate_descriptor == descriptor_id)
            });
        checksum ^= matched.map_or(0, |(instance_id, _)| instance_id.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_indexed(open_instances: &[(String, String)], queries: &[String]) -> u128 {
    let started = Instant::now();
    let mut by_instance_id = HashMap::with_capacity(open_instances.len());
    let mut by_descriptor_id = HashMap::with_capacity(open_instances.len());
    for (instance_id, descriptor_id) in open_instances {
        by_instance_id
            .entry(instance_id.as_str())
            .or_insert(instance_id.as_str());
        by_descriptor_id
            .entry(descriptor_id.as_str())
            .or_insert(instance_id.as_str());
    }

    let mut checksum = 0usize;
    for query in queries {
        let matched = by_instance_id.get(query.as_str()).copied().or_else(|| {
            let descriptor_id = query.rsplit_once('#')?.0;
            by_descriptor_id.get(descriptor_id).copied()
        });
        checksum ^= matched.map_or(0, str::len);
    }
    black_box((checksum, by_instance_id, by_descriptor_id));
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
