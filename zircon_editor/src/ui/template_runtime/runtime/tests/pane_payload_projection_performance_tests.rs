use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn optimization_batch_hl_editor588_non_anchor_panes_move_attributes() {
    let projection = include_str!("../pane_payload_projection.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    let caller = include_str!("../runtime_host/dynamic_control_state.rs");

    assert!(projection.contains("root.attributes.extend(pane_attributes);"));
    assert!(projection.contains("root.attributes.extend(pane_attributes.clone());"));
    assert_eq!(
        caller.matches("inject_pane_projection_attributes(").count(),
        1
    );
    assert!(!caller.contains("let pane_attributes = inject_pane_projection_attributes"));
    assert!(!caller.contains("append_hybrid_slot_anchor_projection"));
}

fn benchmark_attributes(count: usize, prefix: &str) -> BTreeMap<String, String> {
    (0..count)
        .map(|index| {
            (
                format!("{prefix}_attribute_{index:05}"),
                format!(
                    "{prefix}_value_{index:05}_{}",
                    "pane-payload-value".repeat(4)
                ),
            )
        })
        .collect()
}

fn legacy_non_anchor_projection(
    mut root: BTreeMap<String, String>,
    pane_attributes: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    root.extend(pane_attributes.clone());
    root
}

fn moved_non_anchor_projection(
    mut root: BTreeMap<String, String>,
    pane_attributes: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    root.extend(pane_attributes);
    root
}

fn measure_projection(
    root: BTreeMap<String, String>,
    pane_attributes: BTreeMap<String, String>,
    project: fn(BTreeMap<String, String>, BTreeMap<String, String>) -> BTreeMap<String, String>,
) -> u128 {
    let started = Instant::now();
    let projection = project(root, pane_attributes);
    black_box(&projection);
    drop(projection);
    started.elapsed().as_nanos().max(1)
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hl_editor588_non_anchor_pane_attribute_move_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const ROOT_ATTRIBUTES: usize = 64;
    const PANE_ATTRIBUTES: usize = 8_192;

    let root = benchmark_attributes(ROOT_ATTRIBUTES, "root");
    let pane_attributes = benchmark_attributes(PANE_ATTRIBUTES, "pane");
    assert_eq!(
        moved_non_anchor_projection(root.clone(), pane_attributes.clone()),
        legacy_non_anchor_projection(root.clone(), pane_attributes.clone())
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_root = root.clone();
        let legacy_attributes = pane_attributes.clone();
        let optimized_root = root.clone();
        let optimized_attributes = pane_attributes.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure_projection(
                legacy_root,
                legacy_attributes,
                legacy_non_anchor_projection,
            ));
            optimized_samples.push(measure_projection(
                optimized_root,
                optimized_attributes,
                moved_non_anchor_projection,
            ));
        } else {
            optimized_samples.push(measure_projection(
                optimized_root,
                optimized_attributes,
                moved_non_anchor_projection,
            ));
            legacy_samples.push(measure_projection(
                legacy_root,
                legacy_attributes,
                legacy_non_anchor_projection,
            ));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR588_PANE_ATTRIBUTE_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             pane_attributes_per_sample={PANE_ATTRIBUTES} legacy_attribute_map_clones=1 \
             optimized_attribute_map_clones=0 legacy_p95_ns={legacy_p95} \
             optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 60,
        "moved pane attributes P95 {optimized_p95}ns exceeded 60% of cloned P95 {legacy_p95}ns"
    );
}
