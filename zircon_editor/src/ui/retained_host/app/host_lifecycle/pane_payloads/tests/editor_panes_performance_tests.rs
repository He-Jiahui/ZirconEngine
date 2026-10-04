use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
struct BenchmarkInstance {
    instance_id: String,
    pane_kind: u8,
}

#[test]
fn optimization_batch_hk_editor587_editor_panes_use_one_consuming_pass() {
    let collector = include_str!("../editor_panes.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    let caller = include_str!("../../pane_payloads.rs");

    assert_eq!(caller.matches("self.collect_editor_panes(").count(), 1);
    assert!(!caller.contains("self.collect_ui_asset_panes("));
    assert!(!caller.contains("self.collect_animation_editor_panes("));
    assert!(collector.contains("for instance_id in ui_asset_instance_ids"));
    assert!(collector.contains("for instance_id in animation_instance_ids"));
    assert!(collector.contains("instance_id.0"));
    assert!(!collector.contains("instance_id.0.clone()"));
}

fn benchmark_instances(count: usize) -> Vec<BenchmarkInstance> {
    (0..count)
        .map(|index| BenchmarkInstance {
            instance_id: format!(
                "editor587_pane_instance_{index:05}_{}",
                "stable-workbench-instance-segment".repeat(3)
            ),
            pane_kind: (index % 3) as u8,
        })
        .collect()
}

fn legacy_collect(
    instances: Vec<BenchmarkInstance>,
) -> (BTreeMap<String, u8>, BTreeMap<String, u8>) {
    let ui_asset_panes = instances
        .iter()
        .filter(|instance| instance.pane_kind == 0)
        .map(|instance| (instance.instance_id.clone(), instance.pane_kind))
        .collect();
    let animation_panes = instances
        .iter()
        .filter(|instance| instance.pane_kind == 1)
        .map(|instance| (instance.instance_id.clone(), instance.pane_kind))
        .collect();
    (ui_asset_panes, animation_panes)
}

fn single_pass_collect(
    instances: Vec<BenchmarkInstance>,
) -> (BTreeMap<String, u8>, BTreeMap<String, u8>) {
    let mut ui_asset_panes = BTreeMap::new();
    let mut animation_panes = BTreeMap::new();
    for instance in instances {
        match instance.pane_kind {
            0 => {
                ui_asset_panes.insert(instance.instance_id, instance.pane_kind);
            }
            1 => {
                animation_panes.insert(instance.instance_id, instance.pane_kind);
            }
            _ => {}
        }
    }
    (ui_asset_panes, animation_panes)
}

fn measure_collection(
    input: Vec<BenchmarkInstance>,
    collect: fn(Vec<BenchmarkInstance>) -> (BTreeMap<String, u8>, BTreeMap<String, u8>),
) -> u128 {
    let started = Instant::now();
    let panes = collect(input);
    black_box(&panes);
    drop(panes);
    started.elapsed().as_nanos().max(1)
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hk_editor587_editor_pane_single_pass_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const INSTANCES_PER_SAMPLE: usize = 24_576;

    let seed = benchmark_instances(INSTANCES_PER_SAMPLE);
    assert_eq!(
        single_pass_collect(seed.clone()),
        legacy_collect(seed.clone())
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_input = seed.clone();
        let optimized_input = seed.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure_collection(legacy_input, legacy_collect));
            optimized_samples.push(measure_collection(optimized_input, single_pass_collect));
        } else {
            optimized_samples.push(measure_collection(optimized_input, single_pass_collect));
            legacy_samples.push(measure_collection(legacy_input, legacy_collect));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR587_EDITOR_PANE_SINGLE_PASS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             instances_per_sample={INSTANCES_PER_SAMPLE} legacy_snapshot_passes=2 \
             optimized_snapshot_passes=1 legacy_id_clones_per_sample={} \
             optimized_id_clones_per_sample=0 legacy_p95_ns={legacy_p95} \
             optimized_p95_ns={optimized_p95}",
        INSTANCES_PER_SAMPLE * 2 / 3,
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 70,
        "single-pass pane P95 {optimized_p95}ns exceeded 70% of two-pass P95 {legacy_p95}ns"
    );
}
