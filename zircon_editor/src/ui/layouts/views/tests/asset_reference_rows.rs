use std::hint::black_box;
use std::time::Instant;

use super::{
    apply_asset_reference_lists_layout, asset_reference_list_metrics, reference_kind_slot_width,
    reference_kind_slot_widths, sync_asset_reference_lists, AssetReferenceListControls,
    AssetReferenceNodePrototypes, REFERENCE_ROW_NODE_COUNT,
};
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::workbench::snapshot::{
    AssetReferenceSnapshot, AssetSelectionSnapshot, AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};

const LEFT: AssetReferenceListControls = AssetReferenceListControls {
    title_control_id: "LeftTitleText",
    empty_control_id: "LeftEmptyText",
    panel_control_id: "LeftPanel",
    scroll_body_control_id: "LeftScrollBody",
    row_panel_control_id: "LeftRowPanel",
    row_name_control_id: "LeftRowNameText",
    row_locator_control_id: "LeftRowLocatorText",
    row_kind_control_id: "LeftRowKindText",
    node_id_scope: "test.references.left",
    title: "References",
    empty_text: "No direct references",
};
const RIGHT: AssetReferenceListControls = AssetReferenceListControls {
    title_control_id: "RightTitleText",
    empty_control_id: "RightEmptyText",
    panel_control_id: "RightPanel",
    scroll_body_control_id: "RightScrollBody",
    row_panel_control_id: "RightRowPanel",
    row_name_control_id: "RightRowNameText",
    row_locator_control_id: "RightRowLocatorText",
    row_kind_control_id: "RightRowKindText",
    node_id_scope: "test.references.right",
    title: "Used By",
    empty_text: "No usages",
};

#[test]
fn reference_list_metrics_follow_shared_component_tokens() {
    let metrics = asset_reference_list_metrics();
    let density = EditorDensityTokens::workbench_dense();
    let controls = EditorControlTokens::workbench_dense();
    let typography = EditorTypographyTokens::workbench_default();

    assert_eq!(metrics.panel_gap, density.gap_medium);
    assert_eq!(metrics.min_column_width, controls.default_height * 5.0);
    assert_eq!(
        metrics.row_height,
        density.row_height + density.gap_small + controls.border_width * 2.0
    );
    assert_eq!(
        metrics.text_line_height,
        typography.caption_size * typography.line_height
    );
}

#[test]
fn dynamic_reference_rows_resync_from_retained_prototypes() {
    let mut nodes = prototypes();
    let initial = snapshot(
        vec![reference("first", "First", "Content/First")],
        vec![reference("used", "Used", "Content/Used")],
    );
    let refreshed = snapshot(
        vec![
            reference("second", "Second", "Content/Second"),
            reference("third", "Third", "Content/Third"),
        ],
        Vec::new(),
    );

    sync_asset_reference_lists(&mut nodes, &initial, LEFT, RIGHT);
    sync_asset_reference_lists(&mut nodes, &refreshed, LEFT, RIGHT);

    assert_eq!(text(&nodes, "LeftTitleText"), "References (2)");
    assert_eq!(text(&nodes, "LeftRowNameText02"), "Third");
    assert!(node(&nodes, "RightRowPanel01").is_none());
    assert_eq!(text(&nodes, "RightEmptyText"), "No usages");
    let left_prototype = node(&nodes, "LeftRowPanel").expect("retained prototype");
    assert_eq!(left_prototype.frame.width, 0.0);
    assert_eq!(left_prototype.frame.height, 0.0);
    assert!(left_prototype.text.is_empty());
}

#[test]
fn dynamic_reference_rows_reserve_known_node_count_before_append() {
    let mut nodes = prototypes();
    let retained_len = nodes.len();
    let references = (0..128)
        .map(|index| {
            reference(
                &format!("uuid-{index}"),
                &format!("Reference {index}"),
                &format!("Content/Reference{index}"),
            )
        })
        .collect::<Vec<_>>();

    sync_asset_reference_lists(&mut nodes, &snapshot(references, Vec::new()), LEFT, RIGHT);

    assert_eq!(
        nodes.len(),
        retained_len + 128 * REFERENCE_ROW_NODE_COUNT,
        "each reference remains a four-node row",
    );
    assert!(
        nodes.capacity() >= retained_len + 128 * REFERENCE_ROW_NODE_COUNT,
        "known row count should be reserved before extending",
    );
    assert_eq!(text(&nodes, "LeftRowNameText02"), "Reference 1");
    assert_eq!(text(&nodes, "LeftRowKindText128"), "Unknown");
}

#[test]
fn prototype_scan_preserves_first_matching_template_node() {
    let mut nodes = prototypes();
    node_mut(&mut nodes, LEFT.row_name_control_id).text = "first".into();
    nodes.push(ViewTemplateNodeData {
        node_id: "test.references.left.duplicate-name".into(),
        control_id: LEFT.row_name_control_id.into(),
        text: "later".into(),
        ..ViewTemplateNodeData::default()
    });

    let prototypes = AssetReferenceNodePrototypes::from_nodes(&nodes, LEFT)
        .expect("all four reference prototypes");
    assert_eq!(prototypes.name.text, "first");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_editor742_reference_row_capacity_benchmark() {
    const ROWS: usize = 4096;
    const SAMPLES: usize = 11;
    const TARGET_RATIO_PERCENT: u128 = 80;
    let references = (0..ROWS)
        .map(|index| {
            reference(
                &format!("uuid-{index}"),
                &format!("Reference {index}"),
                &format!("Content/Reference{index}"),
            )
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let measure = |reserve: bool| {
            let mut nodes = prototypes();
            let prototypes =
                AssetReferenceNodePrototypes::from_nodes(&nodes, LEFT).expect("prototypes");
            nodes.retain(|node| {
                !super::is_dynamic_reference_row_component(LEFT, node.control_id.as_str())
            });
            let started = Instant::now();
            if reserve {
                nodes.reserve(ROWS * REFERENCE_ROW_NODE_COUNT);
            }
            for (index, reference) in references.iter().enumerate() {
                nodes.extend(prototypes.row_nodes(LEFT, index + 1, reference));
            }
            black_box(nodes);
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "EDITOR742_REFERENCE_ROW_CAPACITY_BENCH_V1 rows={ROWS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={} target_ratio_percent={TARGET_RATIO_PERCENT}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100)
            <= legacy_p95.saturating_mul(TARGET_RATIO_PERCENT),
        "optimized P95 {optimized_p95}ns must be at most {TARGET_RATIO_PERCENT}% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_editor743_reference_prototype_scan_benchmark() {
    const PREFIX_NODES: usize = 4096;
    const SAMPLES: usize = 11;
    const TARGET_RATIO_PERCENT: u128 = 80;
    let mut nodes = Vec::with_capacity(PREFIX_NODES + 16);
    for index in 0..PREFIX_NODES {
        nodes.push(ViewTemplateNodeData {
            node_id: format!("test.references.noise_{index}").into(),
            control_id: format!("ReferenceNoise{index}").into(),
            ..ViewTemplateNodeData::default()
        });
    }
    nodes.extend(prototypes());
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let measure = |optimized: bool| {
            let started = Instant::now();
            if optimized {
                black_box(AssetReferenceNodePrototypes::from_nodes(&nodes, LEFT));
            } else {
                black_box(super::find_node(&nodes, LEFT.row_panel_control_id).cloned());
                black_box(super::find_node(&nodes, LEFT.row_name_control_id).cloned());
                black_box(super::find_node(&nodes, LEFT.row_locator_control_id).cloned());
                black_box(super::find_node(&nodes, LEFT.row_kind_control_id).cloned());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "EDITOR743_REFERENCE_PROTOTYPE_SCAN_BENCH_V1 prefix_nodes={PREFIX_NODES} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={} target_ratio_percent={TARGET_RATIO_PERCENT}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100)
            <= legacy_p95.saturating_mul(TARGET_RATIO_PERCENT),
        "optimized P95 {optimized_p95}ns must be at most {TARGET_RATIO_PERCENT}% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
fn reference_lists_use_columns_then_stack_for_narrow_content() {
    let mut nodes = prototypes();
    nodes.push(frame_node("ReferenceContent", 20.0, 40.0, 520.0, 132.0));
    sync_asset_reference_lists(
        &mut nodes,
        &snapshot(
            vec![reference("left", "Left", "Content/Left")],
            vec![reference("right", "Right", "Content/Right")],
        ),
        LEFT,
        RIGHT,
    );

    apply_asset_reference_lists_layout(&mut nodes, "ReferenceContent", LEFT, RIGHT);
    assert_eq!(node(&nodes, "LeftPanel").expect("left").frame.width, 256.0);
    assert_eq!(node(&nodes, "RightPanel").expect("right").frame.x, 284.0);

    node_mut(&mut nodes, "ReferenceContent").frame.width = 300.0;
    apply_asset_reference_lists_layout(&mut nodes, "ReferenceContent", LEFT, RIGHT);
    assert!(
        node(&nodes, "RightPanel").expect("stacked right").frame.y
            > node(&nodes, "LeftPanel").expect("stacked left").frame.y
    );
}

#[test]
fn kind_slot_uses_runtime_text_width_with_a_relative_budget_cap() {
    let metrics = asset_reference_list_metrics();
    let label = "W".repeat(64);
    let narrow_width = 80.0;
    let wide_width = 320.0;
    let narrow = reference_kind_slot_width(&label, narrow_width, metrics);
    let wide = reference_kind_slot_width(&label, wide_width, metrics);
    assert_eq!(
        narrow,
        (narrow_width - metrics.text_inset * 2.0) * metrics.kind_max_width_fraction
    );
    assert_eq!(
        wide,
        (wide_width - metrics.text_inset * 2.0) * metrics.kind_max_width_fraction
    );
    assert!(wide > narrow);
}

#[test]
fn kind_width_index_is_dense_and_rejects_sparse_control_suffixes() {
    let metrics = asset_reference_list_metrics();
    let mut nodes = prototypes();
    sync_asset_reference_lists(
        &mut nodes,
        &snapshot(
            vec![
                reference("first", "First", "Content/First"),
                reference("second", "Second", "Content/Second"),
            ],
            Vec::new(),
        ),
        LEFT,
        RIGHT,
    );

    let widths = reference_kind_slot_widths(&nodes, LEFT, 320.0, metrics);
    assert_eq!(widths.len(), 2);
    assert!(widths.iter().all(Option::is_some));

    nodes.push(ViewTemplateNodeData {
        node_id: "test.references.left.sparse".into(),
        control_id: "LeftRowKindText999999".into(),
        text: "Sparse".into(),
        ..ViewTemplateNodeData::default()
    });
    let sparse_widths = reference_kind_slot_widths(&nodes, LEFT, 320.0, metrics);
    assert!(sparse_widths.len() <= nodes.len());
}

fn prototypes() -> Vec<ViewTemplateNodeData> {
    [LEFT, RIGHT]
        .into_iter()
        .flat_map(|controls| {
            [
                controls.title_control_id,
                controls.empty_control_id,
                controls.panel_control_id,
                controls.scroll_body_control_id,
                controls.row_panel_control_id,
                controls.row_name_control_id,
                controls.row_locator_control_id,
                controls.row_kind_control_id,
            ]
        })
        .map(|control_id| ViewTemplateNodeData {
            node_id: control_id.into(),
            control_id: control_id.into(),
            ..ViewTemplateNodeData::default()
        })
        .collect()
}

fn snapshot(
    references: Vec<AssetReferenceSnapshot>,
    used_by: Vec<AssetReferenceSnapshot>,
) -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        selection: AssetSelectionSnapshot {
            references,
            used_by,
            ..AssetSelectionSnapshot::default()
        },
        ..AssetWorkspaceSnapshot::default()
    }
}

fn reference(uuid: &str, display_name: &str, locator: &str) -> AssetReferenceSnapshot {
    AssetReferenceSnapshot {
        uuid: uuid.to_string(),
        display_name: display_name.to_string(),
        locator: locator.to_string(),
        ..AssetReferenceSnapshot::default()
    }
}

fn frame_node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        frame: ViewTemplateFrameData {
            x,
            y,
            width,
            height,
        },
        ..ViewTemplateNodeData::default()
    }
}

fn node<'a>(
    nodes: &'a [ViewTemplateNodeData],
    control_id: &str,
) -> Option<&'a ViewTemplateNodeData> {
    nodes.iter().find(|node| node.control_id == control_id)
}

fn node_mut<'a>(
    nodes: &'a mut [ViewTemplateNodeData],
    control_id: &str,
) -> &'a mut ViewTemplateNodeData {
    nodes
        .iter_mut()
        .find(|node| node.control_id == control_id)
        .expect("test node")
}

fn text<'a>(nodes: &'a [ViewTemplateNodeData], control_id: &str) -> &'a str {
    node(nodes, control_id).expect("text node").text.as_str()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
