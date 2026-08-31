use std::hint::black_box;
use std::time::Instant;

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::is_workbench_list_row;

const PERF_MARKER: &str = "EDITOR303_LIST_ROW_TITLE_EARLY_EXIT_BENCH_V1";

#[test]
fn optimization_batch_20260830bf_editor_list_row_title_exclusion_preserves_identity() {
    let title = TemplatePaneNodeData {
        control_id: "WorkbenchListTitle".into(),
        component_role: "list-row".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(!is_workbench_list_row(&title));
    let row = TemplatePaneNodeData {
        control_id: "WorkbenchListEntry".into(),
        component_role: "list-row".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(is_workbench_list_row(&row));
}

#[test]
fn optimization_batch_20260830bf_editor_list_row_title_exclusion_source_contract() {
    let source = include_str!("../identity.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("list-row identity production source");
    assert!(production.contains("if node.control_id.as_str().ends_with(\"Title\")"));
    assert!(production.contains("return false;"));
    assert!(production.contains("is_component_family(node, TemplateComponentFamily::ListRow)"));
    assert!(
        production
            .find("ends_with(\"Title\")")
            .expect("title guard")
            < production
                .find("is_component_family(node")
                .expect("family classification")
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bf_editor_list_row_title_exclusion_p95() {
    const NODES: usize = 10_000;
    const VARIANT_BYTES: usize = 2_048;
    const SAMPLES: usize = 17;
    let node = black_box(TemplatePaneNodeData {
        control_id: "WorkbenchListTitle".into(),
        role: "OrdinaryRole".into(),
        component_role: "ordinary-role".into(),
        component_category: "collection".into(),
        component_layout_role: "grid".into(),
        ..TemplatePaneNodeData::default()
    });
    let long_variant = "x".repeat(VARIANT_BYTES);
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..NODES {
                let node = black_box(&node);
                let control_id = black_box(node.control_id.as_str());
                let role = black_box(node.component_role.as_str());
                let component_category = black_box(node.component_category.as_str());
                let component_layout_role = black_box(node.component_layout_role.as_str());
                let host_role = black_box(node.role.as_str());
                let is_row = if pass == 0 {
                    let family = role == "list-row"
                        || host_role == "ListRow"
                        || (component_category == "collection"
                            && component_layout_role == "virtual-list")
                        || control_id == "ListRow";
                    family && !control_id.ends_with("Title")
                } else {
                    is_workbench_list_row(node)
                };
                checksum += usize::from(is_row) + long_variant.len();
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} nodes={NODES} variant_bytes={VARIANT_BYTES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
