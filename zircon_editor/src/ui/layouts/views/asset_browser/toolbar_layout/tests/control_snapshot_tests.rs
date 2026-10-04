use std::hint::black_box;
use std::time::Instant;

use super::{node_frame, ToolbarControlSnapshot, ViewTemplateNodeData};

#[test]
fn toolbar_control_snapshot_preserves_first_match_and_missing_dropdown() {
    let mut nodes = toolbar_fixture(32);
    let final_control = nodes
        .iter()
        .position(|node| node.control_id == "ImportModel")
        .unwrap();
    nodes.insert(
        final_control,
        ViewTemplateNodeData {
            control_id: "AssetBrowserKindFilterDropdown".into(),
            frame: super::ViewTemplateFrameData {
                width: 999.0,
                ..Default::default()
            },
            ..ViewTemplateNodeData::default()
        },
    );
    nodes.insert(
        final_control + 1,
        ViewTemplateNodeData {
            control_id: "FixtureUnknownControl".into(),
            ..ViewTemplateNodeData::default()
        },
    );
    assert_eq!(
        ToolbarControlSnapshot::from_nodes(&nodes),
        legacy_snapshot(&nodes)
    );
    assert_eq!(
        ToolbarControlSnapshot::from_nodes(&nodes).filter_width,
        Some(168.0)
    );

    let first_dropdown = nodes
        .iter_mut()
        .find(|node| node.control_id == "AssetBrowserKindFilterDropdown")
        .unwrap();
    first_dropdown.frame.width = 0.0;
    let zero_width = ToolbarControlSnapshot::from_nodes(&nodes);
    assert_eq!(zero_width, legacy_snapshot(&nodes));
    assert_eq!(super::width_or(zero_width.filter_width, 168.0), 168.0);
    let mut laid_out = nodes.clone();
    assert!(super::apply_asset_browser_toolbar_layout(&mut laid_out, 1_600.0).is_some());
    let dropdown_frames = laid_out
        .iter()
        .filter(|node| node.control_id == "AssetBrowserKindFilterDropdown")
        .map(|node| &node.frame)
        .collect::<Vec<_>>();
    assert_eq!(dropdown_frames.len(), 2);
    assert!(dropdown_frames.iter().all(|frame| frame.width == 168.0));

    nodes.retain(|node| node.control_id != "AssetBrowserKindFilterDropdown");
    let missing = ToolbarControlSnapshot::from_nodes(&nodes);
    assert_eq!(missing, legacy_snapshot(&nodes));
    assert_eq!(missing.filter_width, None);
}

#[test]
#[ignore = "Windows Release active dropdown toolbar read-projection gate"]
fn editor57_toolbar_control_snapshot_release_benchmark() {
    const SAMPLE_PAIRS: usize = 17;
    const ITERATIONS_PER_SAMPLE: usize = 256;
    let small = toolbar_fixture(64);
    let large = toolbar_fixture(4_096);
    let small_p95 = benchmark_case(
        "small_node_table",
        &small,
        SAMPLE_PAIRS,
        ITERATIONS_PER_SAMPLE,
    );
    let large_p95 = benchmark_case(
        "large_node_table",
        &large,
        SAMPLE_PAIRS,
        ITERATIONS_PER_SAMPLE,
    );
    for (workload, (legacy_p95, optimized_p95), limit_percent) in [
        ("small_node_table", small_p95, 110_u128),
        ("large_node_table", large_p95, 70_u128),
    ] {
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(limit_percent),
            "{workload} toolbar read p95 {optimized_p95}ns exceeded {limit_percent}% of legacy {legacy_p95}ns"
        );
    }
}

fn toolbar_fixture(prefix_count: usize) -> Vec<ViewTemplateNodeData> {
    let mut nodes = (0..prefix_count)
        .map(|index| ViewTemplateNodeData {
            control_id: format!("FixturePrefix{index:05}").into(),
            ..ViewTemplateNodeData::default()
        })
        .collect::<Vec<_>>();
    nodes.extend(
        [
            ("AssetBrowserToolbarPanel", 900.0),
            ("AssetBrowserImportPanel", 900.0),
            ("AssetBrowserViewModeListButton", 32.0),
            ("AssetBrowserViewModeThumbButton", 32.0),
            ("LocateSelectedAsset", 32.0),
            ("AssetBrowserKindFilterDropdown", 168.0),
            ("ImportModel", 96.0),
        ]
        .into_iter()
        .map(|(control_id, width)| ViewTemplateNodeData {
            control_id: control_id.into(),
            frame: super::ViewTemplateFrameData {
                width,
                ..Default::default()
            },
            ..ViewTemplateNodeData::default()
        }),
    );
    nodes
}

fn legacy_snapshot(nodes: &[ViewTemplateNodeData]) -> ToolbarControlSnapshot {
    let toolbar = node_frame(nodes, "AssetBrowserToolbarPanel");
    let import_panel = node_frame(nodes, "AssetBrowserImportPanel");
    let locate_width = node_frame(nodes, "LocateSelectedAsset").map(|frame| frame.width);
    let filter_width = node_frame(nodes, "AssetBrowserKindFilterDropdown").map(|frame| frame.width);
    let list_width = node_frame(nodes, "AssetBrowserViewModeListButton").map(|frame| frame.width);
    let thumb_width = node_frame(nodes, "AssetBrowserViewModeThumbButton").map(|frame| frame.width);
    let import_button_width = node_frame(nodes, "ImportModel").map(|frame| frame.width);
    black_box(
        nodes
            .iter()
            .any(|node| node.control_id == "AssetBrowserKindFilterDropdown"),
    );
    ToolbarControlSnapshot {
        toolbar,
        import_panel,
        locate_width,
        filter_width,
        list_width,
        thumb_width,
        import_button_width,
    }
}

fn benchmark_case(
    workload: &str,
    nodes: &[ViewTemplateNodeData],
    sample_pairs: usize,
    iterations_per_sample: usize,
) -> (u128, u128) {
    assert_eq!(
        legacy_snapshot(nodes),
        ToolbarControlSnapshot::from_nodes(nodes)
    );
    let mut legacy_samples = Vec::with_capacity(sample_pairs);
    let mut optimized_samples = Vec::with_capacity(sample_pairs);
    for pair in 0..sample_pairs {
        if pair % 2 == 0 {
            legacy_samples.push(measure(nodes, iterations_per_sample, legacy_snapshot));
            optimized_samples.push(measure(
                nodes,
                iterations_per_sample,
                ToolbarControlSnapshot::from_nodes,
            ));
        } else {
            optimized_samples.push(measure(
                nodes,
                iterations_per_sample,
                ToolbarControlSnapshot::from_nodes,
            ));
            legacy_samples.push(measure(nodes, iterations_per_sample, legacy_snapshot));
        }
    }
    let legacy_p95 = percentile_ns(&legacy_samples, 95);
    let optimized_p95 = percentile_ns(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR57_ACTIVE_DROPDOWN_TOOLBAR_READ_BENCH_V1 workload={workload} nodes={} sample_pairs={sample_pairs} iterations_per_sample={iterations_per_sample} legacy_p50_ns={} legacy_p95_ns={legacy_p95} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={optimized_p95} optimized_p99_ns={} threshold_p95_ratio={}",
        nodes.len(),
        percentile_ns(&legacy_samples, 50),
        percentile_ns(&legacy_samples, 99),
        percentile_ns(&optimized_samples, 50),
        percentile_ns(&optimized_samples, 99),
        if workload == "small_node_table" { "1.10" } else { "0.70" }
    );
    (legacy_p95, optimized_p95)
}

fn measure(
    nodes: &[ViewTemplateNodeData],
    iterations: usize,
    operation: fn(&[ViewTemplateNodeData]) -> ToolbarControlSnapshot,
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
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}
