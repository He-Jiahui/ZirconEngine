use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};
use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

#[test]
fn asset_browser_toolbar_metrics_project_from_dense_design_tokens() {
    let mut density = EditorDensityTokens::workbench_dense();
    density.row_height = 28.0;
    density.gap_small = 5.0;
    density.gap_medium = 10.0;
    let mut controls = EditorControlTokens::workbench_dense();
    controls.border_width = 2.0;
    controls.compact_height = 31.0;

    let metrics = asset_browser_toolbar_metrics_from_tokens(density, controls);

    assert_eq!(metrics.toolbar_height, 38.0);
    assert_eq!(metrics.control_height, 34.0);
    assert_eq!(metrics.control_offset_y, 2.0);
    assert_eq!(metrics.compact_icon_width, 31.0);
    assert_eq!(metrics.side_pad, 10.0);
    assert_eq!(metrics.root_gap, 6.0);
    assert_eq!(metrics.group_gap, 10.0);
    assert_eq!(metrics.group_frame_pad, 3.0);
    assert_eq!(metrics.row_gap, 5.0);
    assert_eq!(metrics.view_button_gap, 5.0);

    assert!(!chip_fits_in_width(0.0, false, 44.0, 0.0, metrics));
    assert!(chip_fits_in_width(0.0, false, 44.0, 44.0, metrics));
    assert!(!chip_fits_in_width(44.0, true, 78.0, 126.0, metrics));
    assert!(chip_fits_in_width(44.0, true, 78.0, 127.0, metrics));
}

#[test]
fn linear_kind_chip_selection_preserves_legacy_visibility() {
    let metrics = asset_browser_toolbar_metrics();
    for selected in [None, Some("AssetBrowserKindShaderChip")] {
        for missing_last_chip in [false, true] {
            let mut nodes = kind_chip_fixture(selected, 0);
            if missing_last_chip {
                nodes.pop();
            }
            for width_limit in [0.0, 44.0, 130.0, 260.0, 1_000.0] {
                assert_eq!(
                    select_visible_kind_chips(&nodes, width_limit, metrics).visible,
                    legacy_visible_kind_chips(&nodes, width_limit, metrics),
                    "visibility must match for selected={selected:?}, missing_last_chip={missing_last_chip}, width={width_limit}"
                );
            }
        }
    }
}

#[test]
fn linear_kind_chip_selection_avoids_candidate_clones_and_repeated_width_scans() {
    let source = include_str!("../toolbar_layout.rs");
    let implementation = source
        .split("mod kind_chip_single_scan_tests;")
        .next()
        .unwrap_or(source);

    assert!(!implementation.contains("let mut candidate = visible.clone()"));
    assert!(!implementation.contains("fn chip_stack_width("));
    assert!(implementation.contains("std::array::from_fn"));
    assert!(implementation.contains("visible: [bool; KIND_CHIP_COUNT]"));
}

#[test]
#[ignore = "release-only asset toolbar chip selection performance gate"]
fn linear_kind_chip_selection_release_benchmark() {
    const SAMPLE_COUNT: usize = 11;
    const ITERATIONS_PER_SAMPLE: usize = 8_192;
    const PREFIX_NODE_COUNT: usize = 64;
    const MAX_OPTIMIZED_TO_LEGACY_PERCENT: u128 = 40;

    let nodes = kind_chip_fixture(None, PREFIX_NODE_COUNT);
    let metrics = asset_browser_toolbar_metrics();
    let width_limit = 1_000.0;
    black_box(select_visible_kind_chips(&nodes, width_limit, metrics));
    black_box(legacy_visible_kind_chips(&nodes, width_limit, metrics));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure_kind_chip_selection(
                &nodes,
                metrics,
                width_limit,
                ITERATIONS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_kind_chip_selection(
                &nodes,
                metrics,
                width_limit,
                ITERATIONS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_kind_chip_selection(
                &nodes,
                metrics,
                width_limit,
                ITERATIONS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_kind_chip_selection(
                &nodes,
                metrics,
                width_limit,
                ITERATIONS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95_ns = duration_p95_ns(legacy_samples);
    let optimized_p95_ns = duration_p95_ns(optimized_samples);
    let reduction_basis_points = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(10_000)
        / legacy_p95_ns.max(1);
    println!(
        "EDITOR57_LINEAR_KIND_CHIP_SELECTION_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} reduction_basis_points={reduction_basis_points} samples={SAMPLE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} prefix_nodes={PREFIX_NODE_COUNT} chips={KIND_CHIP_COUNT} node_searches_per_selection=32->1 candidate_vec_clones_per_selection=5->0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(MAX_OPTIMIZED_TO_LEGACY_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {MAX_OPTIMIZED_TO_LEGACY_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn measure_kind_chip_selection(
    nodes: &[ViewTemplateNodeData],
    metrics: AssetBrowserToolbarMetrics,
    width_limit: f32,
    iterations: usize,
    optimized: bool,
) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        if optimized {
            black_box(select_visible_kind_chips(
                black_box(nodes),
                width_limit,
                metrics,
            ));
        } else {
            black_box(legacy_visible_kind_chips(
                black_box(nodes),
                width_limit,
                metrics,
            ));
        }
    }
    started.elapsed()
}

fn duration_p95_ns(mut samples: Vec<Duration>) -> u128 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index].as_nanos()
}

fn legacy_visible_kind_chips(
    nodes: &[ViewTemplateNodeData],
    width_limit: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> [bool; KIND_CHIP_COUNT] {
    let selected_chip = KIND_CHIPS
        .iter()
        .find(|(control_id, _)| is_selected(nodes, control_id))
        .map(|(control_id, _)| *control_id);
    let mut visible = Vec::new();
    for &(control_id, _) in KIND_CHIPS {
        if control_id == "AssetBrowserKindAllChip" || Some(control_id) == selected_chip {
            visible.push(control_id);
        }
    }
    for &(control_id, _) in KIND_CHIPS {
        if visible.contains(&control_id) {
            continue;
        }
        let mut candidate = visible.clone();
        candidate.push(control_id);
        if legacy_chip_stack_width(nodes, &candidate, metrics) <= width_limit {
            visible.push(control_id);
        }
    }

    std::array::from_fn(|index| visible.contains(&KIND_CHIPS[index].0))
}

fn legacy_chip_stack_width(
    nodes: &[ViewTemplateNodeData],
    control_ids: &[&str],
    metrics: AssetBrowserToolbarMetrics,
) -> f32 {
    let mut width = 0.0;
    let mut visible_count = 0;
    for &(control_id, fallback_width) in KIND_CHIPS {
        if control_ids.contains(&control_id) {
            if visible_count > 0 {
                width += metrics.row_gap;
            }
            width += control_width(nodes, control_id, fallback_width);
            visible_count += 1;
        }
    }
    width
}

fn kind_chip_fixture(
    selected: Option<&str>,
    prefix_node_count: usize,
) -> Vec<ViewTemplateNodeData> {
    let mut nodes = (0..prefix_node_count)
        .map(|index| ViewTemplateNodeData {
            control_id: format!("FixturePrefix{index:03}").into(),
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
