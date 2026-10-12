use std::hint::black_box;
use std::time::Instant;

use super::{
    kind_chip_states, kind_chip_states_from_iter, ViewTemplateNodeData, KIND_CHIPS, KIND_CHIP_COUNT,
};

#[test]
fn kind_chip_state_scan_preserves_first_match_and_missing_fallback() {
    let mut nodes = kind_chip_fixture(Some("AssetBrowserKindShaderChip"), 32);
    let texture = nodes
        .iter_mut()
        .find(|node| node.control_id == "AssetBrowserKindTextureChip")
        .unwrap();
    texture.frame.width = 0.0;
    let final_chip = nodes
        .iter()
        .position(|node| node.control_id == "AssetBrowserKindShaderChip")
        .unwrap();
    nodes.insert(
        final_chip,
        ViewTemplateNodeData {
            control_id: "AssetBrowserKindTextureChip".into(),
            selected: true,
            ..ViewTemplateNodeData::default()
        },
    );
    nodes.insert(
        final_chip + 1,
        ViewTemplateNodeData {
            control_id: "AssetBrowserKindUnknownChip".into(),
            selected: true,
            ..ViewTemplateNodeData::default()
        },
    );
    nodes.push(ViewTemplateNodeData {
        control_id: "FixtureAfterAllChips".into(),
        ..ViewTemplateNodeData::default()
    });
    let mut visited = 0;
    let states = kind_chip_states_from_iter(nodes.iter().inspect(|_| visited += 1));
    assert_eq!(visited, nodes.len() - 1);
    assert_eq!(states, kind_chip_states(&nodes));
    assert_eq!(states, legacy_kind_chip_states(&nodes));
    assert_eq!(states[1], (78.0, false));
    assert_eq!(states[5], (72.0, true));

    nodes.retain(|node| node.control_id != "AssetBrowserKindShaderChip");
    let missing_states = kind_chip_states(&nodes);
    assert_eq!(missing_states, legacy_kind_chip_states(&nodes));
    assert_eq!(missing_states[5], (72.0, false));
}

#[test]
#[ignore = "Windows Release large Asset Browser chip state evidence"]
fn editor57_kind_chip_single_scan_release_benchmark() {
    const PREFIX_NODES: usize = 4_096;
    const ITERATIONS_PER_SAMPLE: usize = 256;
    const SAMPLE_COUNT: usize = 21;
    let nodes = kind_chip_fixture(Some("AssetBrowserKindShaderChip"), PREFIX_NODES);
    assert_eq!(legacy_kind_chip_states(&nodes), kind_chip_states(&nodes));
    black_box(legacy_kind_chip_states(&nodes));
    black_box(kind_chip_states(&nodes));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure_kind_chip_states(
                &nodes,
                ITERATIONS_PER_SAMPLE,
                legacy_kind_chip_states,
            ));
            optimized_samples.push(measure_kind_chip_states(
                &nodes,
                ITERATIONS_PER_SAMPLE,
                kind_chip_states,
            ));
        } else {
            optimized_samples.push(measure_kind_chip_states(
                &nodes,
                ITERATIONS_PER_SAMPLE,
                kind_chip_states,
            ));
            legacy_samples.push(measure_kind_chip_states(
                &nodes,
                ITERATIONS_PER_SAMPLE,
                legacy_kind_chip_states,
            ));
        }
    }
    let legacy_p50 = percentile_ns(&legacy_samples, 50);
    let legacy_p95 = percentile_ns(&legacy_samples, 95);
    let legacy_p99 = percentile_ns(&legacy_samples, 99);
    let optimized_p50 = percentile_ns(&optimized_samples, 50);
    let optimized_p95 = percentile_ns(&optimized_samples, 95);
    let optimized_p99 = percentile_ns(&optimized_samples, 99);
    println!(
        "PERF_RESULT EDITOR57_KIND_CHIP_SINGLE_SCAN_BENCH_V1 prefix_nodes={PREFIX_NODES} chips={KIND_CHIP_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} samples={SAMPLE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99} threshold_p95_ratio=0.70"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "single-scan chip state p95 {optimized_p95}ns exceeded 70% of six-scan baseline {legacy_p95}ns"
    );
}

fn kind_chip_fixture(
    selected: Option<&str>,
    prefix_node_count: usize,
) -> Vec<ViewTemplateNodeData> {
    let mut nodes = (0..prefix_node_count)
        .map(|index| ViewTemplateNodeData {
            control_id: format!("FixturePrefix{index:05}").into(),
            ..ViewTemplateNodeData::default()
        })
        .collect::<Vec<_>>();
    nodes.extend(KIND_CHIPS.iter().map(|&(control_id, width)| {
        let mut node = ViewTemplateNodeData {
            control_id: control_id.into(),
            selected: selected == Some(control_id),
            ..ViewTemplateNodeData::default()
        };
        node.frame.width = width;
        node
    }));
    nodes
}

fn legacy_kind_chip_states(nodes: &[ViewTemplateNodeData]) -> [(f32, bool); KIND_CHIP_COUNT] {
    std::array::from_fn(|index| {
        let (control_id, fallback_width) = KIND_CHIPS[index];
        let Some(node) = nodes.iter().find(|node| node.control_id == control_id) else {
            return (fallback_width, false);
        };
        let width = (node.frame.width > 0.0)
            .then_some(node.frame.width)
            .unwrap_or(fallback_width);
        (width, node.selected)
    })
}

fn measure_kind_chip_states(
    nodes: &[ViewTemplateNodeData],
    iterations: usize,
    operation: fn(&[ViewTemplateNodeData]) -> [(f32, bool); KIND_CHIP_COUNT],
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(operation(black_box(nodes)));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile_ns(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
