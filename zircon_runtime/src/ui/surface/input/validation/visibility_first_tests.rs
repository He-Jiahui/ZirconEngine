use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::{UiTreeNode, UiVisibility},
};

use super::{input_owner_node_is_valid, is_valid_input_owner};
use crate::ui::surface::surface::UiSurface;

const PERF_MARKER: &str = "RUNTIME363_INPUT_OWNER_VISIBILITY_FIRST_BENCH_V1";

#[test]
fn optimization_batch_20260830bk_runtime_input_owner_visibility_preserves_results() {
    let node_id = UiNodeId::new(1);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.input-owner.visibility"));
    surface.tree.insert_root(
        UiTreeNode::new(node_id, UiNodePath::new("root")).with_visibility(UiVisibility::Hidden),
    );
    assert!(!is_valid_input_owner(&surface, node_id));

    let mut disabled = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("disabled"));
    disabled.state_flags.enabled = false;
    assert!(!input_owner_node_is_valid(
        &surface,
        disabled.node_id,
        &disabled
    ));

    let visible = UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("visible"));
    assert!(input_owner_node_is_valid(
        &surface,
        visible.node_id,
        &visible
    ));
}

#[test]
fn optimization_batch_20260830bk_runtime_input_owner_visibility_source_contract() {
    let source = include_str!("../validation.rs");
    assert!(source.contains("node.is_render_visible()"));
    assert!(source.contains("!ui_surface_node_disabled"));
    assert!(source.contains("fn input_owner_node_is_valid"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bk_runtime_input_owner_visibility_p95() {
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let surface = black_box(UiSurface::new(UiTreeId::new("runtime.input-owner.bench")));
    let node = black_box(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("hidden"))
            .with_visibility(UiVisibility::Hidden),
    );
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let valid = if pass == 0 {
                    !super::ui_surface_node_disabled(
                        &surface,
                        node.node_id,
                        &node,
                        node.template_metadata.as_ref(),
                    ) && node.is_render_visible()
                } else {
                    input_owner_node_is_valid(&surface, node.node_id, &node)
                };
                checksum += usize::from(valid);
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
        "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
