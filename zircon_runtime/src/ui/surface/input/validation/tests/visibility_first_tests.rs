use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiInputEvent, UiInputEventMetadata, UiKeyboardInputEvent,
        UiKeyboardInputState,
    },
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::{UiTreeNode, UiVisibility},
};

use super::{input_owner_node_is_valid, is_valid_input_owner, valid_input_owner_route};
use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use crate::ui::surface::surface::UiSurface;
use crate::ui::tree::UiRuntimeTreeRoutingExt;

const PERF_MARKER: &str = "RUNTIME363_INPUT_OWNER_VISIBILITY_FIRST_BENCH_V1";

// Exact HEAD is_valid_input_owner parent walk, before the visitor refactor.
fn legacy_is_valid_input_owner(surface: &UiSurface, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = surface.tree.nodes.get(&id) else {
            return false;
        };
        if !input_owner_node_is_valid(surface, id, node) {
            return false;
        }
        current = node.parent;
    }
    true
}

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
fn valid_input_owner_route_matches_tree_order_and_rejects_invalid_ancestors() {
    let root = UiNodeId::new(1);
    let child = UiNodeId::new(2);
    let grandchild = UiNodeId::new(3);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.input-owner.route"));
    surface
        .tree
        .insert_root(UiTreeNode::new(root, UiNodePath::new("root")));
    surface
        .tree
        .insert_child(root, UiTreeNode::new(child, UiNodePath::new("child")))
        .expect("root exists");
    surface
        .tree
        .insert_child(
            child,
            UiTreeNode::new(grandchild, UiNodePath::new("grandchild")),
        )
        .expect("child exists");

    let route = valid_input_owner_route(&surface, grandchild).expect("visible route");
    assert!(legacy_is_valid_input_owner(&surface, grandchild));
    assert_eq!(route, vec![grandchild, child, root]);
    assert_eq!(
        route,
        surface.tree.bubble_route(grandchild).expect("tree route")
    );

    surface
        .tree
        .nodes
        .get_mut(&child)
        .expect("child")
        .visibility = UiVisibility::Hidden;
    assert!(!is_valid_input_owner(&surface, grandchild));
    assert!(!legacy_is_valid_input_owner(&surface, grandchild));
    assert_eq!(valid_input_owner_route(&surface, grandchild), None);
    assert!(!legacy_is_valid_input_owner(&surface, UiNodeId::new(99)));
    assert_eq!(valid_input_owner_route(&surface, UiNodeId::new(99)), None);
}

#[test]
fn valid_input_owner_route_preserves_deep_routes_after_inline_capacity() {
    const DEPTH: u64 = 80;
    let mut surface = UiSurface::new(UiTreeId::new("runtime.input-owner.deep-route"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
    for id in 2..=DEPTH {
        surface
            .tree
            .insert_child(
                UiNodeId::new(id - 1),
                UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("node-{id}"))),
            )
            .expect("parent exists");
    }
    let target = UiNodeId::new(DEPTH);
    assert_eq!(
        valid_input_owner_route(&surface, target),
        Some(surface.tree.bubble_route(target).expect("tree route"))
    );
}

#[test]
fn hidden_ancestor_rejects_keyboard_dispatch_without_recording_focused_input() {
    let root = UiNodeId::new(1);
    let target = UiNodeId::new(2);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.input-owner.keyboard-rejection"));
    surface
        .tree
        .insert_root(UiTreeNode::new(root, UiNodePath::new("root")));
    let mut focused_node = UiTreeNode::new(target, UiNodePath::new("target"));
    focused_node.state_flags.focusable = true;
    surface
        .tree
        .insert_child(root, focused_node)
        .expect("root exists");
    surface
        .focus_node(target)
        .expect("visible target is focusable");
    surface.tree.nodes.get_mut(&root).expect("root").visibility = UiVisibility::Hidden;

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: UiInputEventMetadata::default(),
                state: UiKeyboardInputState::Pressed,
                key_code: 65,
                scan_code: None,
                physical_key: "KeyA".to_string(),
                logical_key: "KeyA".to_string(),
                text: None,
            }),
        )
        .expect("invalid focused owner returns a dispatch result");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(!result.diagnostics.routed);
    assert_eq!(result.diagnostics.route_target, None);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "owner route rejected"));
    assert!(surface.focus.focused_inputs.is_empty());
}

#[test]
#[ignore = "release-only input route performance evidence"]
fn valid_input_owner_route_release_p95() {
    const DEPTH: usize = 64;
    const ROUTES_PER_SAMPLE: usize = 4_096;
    const SAMPLES: usize = 17;
    const MARKER: &str = "RUNTIME200_VALID_INPUT_OWNER_ROUTE_BENCH_V1";

    fn measure(surface: &UiSurface, target: UiNodeId, fused: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..ROUTES_PER_SAMPLE {
            let surface = black_box(surface);
            let route = if fused {
                valid_input_owner_route(surface, target).expect("valid target")
            } else {
                assert!(legacy_is_valid_input_owner(surface, target));
                surface.tree.bubble_route(target).expect("valid route")
            };
            black_box(route);
        }
        started.elapsed().as_nanos()
    }

    let mut surface = UiSurface::new(UiTreeId::new("runtime.input-owner.route-bench"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
    for id in 2..=DEPTH {
        surface
            .tree
            .insert_child(
                UiNodeId::new((id - 1) as u64),
                UiTreeNode::new(
                    UiNodeId::new(id as u64),
                    UiNodePath::new(format!("node-{id}")),
                ),
            )
            .expect("parent exists");
    }
    let target = UiNodeId::new(DEPTH as u64);
    assert!(legacy_is_valid_input_owner(&surface, target));
    assert_eq!(
        valid_input_owner_route(&surface, target),
        Some(surface.tree.bubble_route(target).expect("tree route"))
    );

    for _ in 0..4 {
        black_box(measure(&surface, target, false));
        black_box(measure(&surface, target, true));
    }
    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut fused = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            legacy.push(measure(&surface, target, false));
            fused.push(measure(&surface, target, true));
        } else {
            fused.push(measure(&surface, target, true));
            legacy.push(measure(&surface, target, false));
        }
    }
    legacy.sort_unstable();
    fused.sort_unstable();
    let p95_index = (SAMPLES * 95).div_ceil(100) - 1;
    let legacy_p95 = legacy[p95_index];
    let fused_p95 = fused[p95_index];
    println!("{MARKER} legacy_p95_ns={legacy_p95} fused_p95_ns={fused_p95}");
    assert!(
        fused_p95.saturating_mul(5) <= legacy_p95.saturating_mul(4),
        "fused route P95 must be at least 20% faster"
    );
}

#[test]
fn optimization_batch_20260830bk_runtime_input_owner_visibility_source_contract() {
    let source = include_str!("../../validation.rs");
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
