use std::hint::black_box;
use std::time::Instant;

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::is_icon_node;

const PERF_MARKER: &str = "EDITOR304_ICON_NAME_FIRST_BENCH_V1";

#[test]
fn optimization_batch_20260830bg_editor_icon_name_first_preserves_identity() {
    let icon = TemplatePaneNodeData {
        icon_name: "toolbar/save.svg".into(),
        role: "Button".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(is_icon_node(&icon));
    let role_icon = TemplatePaneNodeData {
        role: "SvgIcon".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(is_icon_node(&role_icon));
    let ordinary = TemplatePaneNodeData {
        role: "Button".into(),
        ..TemplatePaneNodeData::default()
    };
    assert!(!is_icon_node(&ordinary));
}

#[test]
fn optimization_batch_20260830bg_editor_icon_name_first_source_contract() {
    let source = include_str!("../identity.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("icon identity production source");
    assert!(production.contains("!node.icon_name.is_empty()"));
    assert!(
        production
            .find("!node.icon_name.is_empty()")
            .expect("icon guard")
            < production.find("matches!(node.role").expect("role check")
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bg_editor_icon_name_first_p95() {
    const NODES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let node = black_box(TemplatePaneNodeData {
        role: "ordinary-role".into(),
        icon_name: "toolbar/save.svg".into(),
        ..TemplatePaneNodeData::default()
    });
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..NODES {
                let node = black_box(&node);
                let role = black_box(node.role.as_str());
                let icon_name = black_box(node.icon_name.as_str());
                let matched = if pass == 0 {
                    matches!(role, "Icon" | "IconButton" | "SvgIcon") || !icon_name.is_empty()
                } else {
                    is_icon_node(node)
                };
                checksum += usize::from(matched);
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
        "{PERF_MARKER} nodes={NODES} role_bytes={} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}",
        node.role.len()
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
