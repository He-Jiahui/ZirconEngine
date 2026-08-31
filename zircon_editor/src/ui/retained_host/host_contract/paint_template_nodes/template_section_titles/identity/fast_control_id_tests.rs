use std::hint::black_box;
use std::time::Instant;

use super::is_workbench_section_title;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const PERF_MARKER: &str = "EDITOR302_SECTION_TITLE_CONTROL_ID_FIRST_BENCH_V1";

#[test]
fn optimization_batch_20260830be_editor_section_title_control_id_first_preserves_identity() {
    for control_id in [
        "WorkbenchSectionTitleRoot",
        "WorkbenchTransformLabel",
        "WorkbenchMeshLabel",
    ] {
        let node = TemplatePaneNodeData {
            control_id: control_id.into(),
            component_variant: "unrelated variant tokens".into(),
            ..TemplatePaneNodeData::default()
        };
        assert!(is_workbench_section_title(&node));
    }
    let variant_node = TemplatePaneNodeData {
        component_variant: "surface section-title dense".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(is_workbench_section_title(&variant_node));
    let ordinary_node = TemplatePaneNodeData {
        control_id: "WorkbenchOtherLabel".into(),
        component_variant: "surface dense".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(!is_workbench_section_title(&ordinary_node));
}

#[test]
fn optimization_batch_20260830be_editor_section_title_control_id_first_source_contract() {
    let source = include_str!("../identity.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("section title identity production source");
    assert!(production.contains("if matches!("));
    assert!(production.contains("return true;"));
    assert!(production.contains("split_ascii_whitespace()"));
    assert!(
        production.find("if matches!(").expect("control-id guard")
            < production
                .find("split_ascii_whitespace()")
                .expect("variant scan")
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830be_editor_section_title_control_id_first_p95() {
    const NODES: usize = 10_000;
    const VARIANT_TOKENS: usize = 32;
    const SAMPLES: usize = 17;
    let variant = (0..VARIANT_TOKENS)
        .map(|index| format!("variant-{index}"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut optimized = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..NODES {
                let control_id = "WorkbenchMeshLabel";
                let is_title = if pass == 0 {
                    variant
                        .split_ascii_whitespace()
                        .any(|token| token == "section-title")
                        || matches!(
                            control_id,
                            "WorkbenchSectionTitleRoot"
                                | "WorkbenchTransformLabel"
                                | "WorkbenchMeshLabel"
                        )
                } else if matches!(
                    control_id,
                    "WorkbenchSectionTitleRoot" | "WorkbenchTransformLabel" | "WorkbenchMeshLabel"
                ) {
                    true
                } else {
                    variant
                        .split_ascii_whitespace()
                        .any(|token| token == "section-title")
                };
                checksum += usize::from(is_title);
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                legacy.push(elapsed);
            } else {
                optimized.push(elapsed);
            }
        }
    }
    legacy.sort_unstable();
    optimized.sort_unstable();
    let legacy_p95 = legacy[(SAMPLES * 95).div_ceil(100) - 1];
    let optimized_p95 = optimized[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * legacy_p95.saturating_sub(optimized_p95) as f64 / legacy_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} nodes={NODES} variant_tokens={VARIANT_TOKENS} samples={SAMPLES} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(optimized_p95.saturating_mul(10) <= legacy_p95.saturating_mul(7));
}
