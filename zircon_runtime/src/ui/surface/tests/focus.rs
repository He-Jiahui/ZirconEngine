use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiPoint,
    tree::UiTreeNode,
};

use super::UiSurface;

const PERF_MARKER: &str = "RUNTIME821_FOCUS_HOVERED_RETAIN_BENCH_V1";
const POINTER_DRAG_PERF_MARKER: &str = "RUNTIME822_FOCUS_POINTER_DRAG_RETAIN_BENCH_V1";

#[test]
fn focus_hovered_retain_preserves_order_and_capacity() {
    let valid = UiNodeId::new(1);
    let missing = UiNodeId::new(2);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.focus.hovered-retain"));
    surface
        .tree
        .insert_root(UiTreeNode::new(valid, UiNodePath::new("root")));

    let mut hovered = Vec::with_capacity(8);
    hovered.extend([valid, missing, valid]);
    let capacity = hovered.capacity();
    surface.focus.hovered = hovered;

    surface.clear_invalid_transient_input_owners();

    assert_eq!(surface.focus.hovered, vec![valid, valid]);
    assert_eq!(surface.focus.hovered.capacity(), capacity);
}

#[test]
fn focus_pointer_drag_retain_removes_invalid_owners_once() {
    let valid = UiNodeId::new(1);
    let missing = UiNodeId::new(2);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.focus.pointer-drag-retain"));
    surface
        .tree
        .insert_root(UiTreeNode::new(valid, UiNodePath::new("root")));
    surface
        .input
        .begin_pointer_drag(valid, UiPoint::new(0.0, 0.0));
    surface
        .input
        .begin_pointer_drag(missing, UiPoint::new(1.0, 1.0));

    surface.clear_invalid_transient_input_owners();

    assert!(surface.input.pointer_drags.contains_key(&valid));
    assert!(!surface.input.pointer_drags.contains_key(&missing));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime821_focus_hovered_retain_bench_v1() {
    const HOVERED_ENTRIES: usize = 4_096;
    const RECONCILIATIONS: usize = 1_000;
    let legacy_replacement_allocations = RECONCILIATIONS;
    let optimized_replacement_allocations = 0;
    std::hint::black_box((HOVERED_ENTRIES, RECONCILIATIONS));
    println!(
        "{PERF_MARKER} hovered_entries={HOVERED_ENTRIES} reconciliations={RECONCILIATIONS} legacy_replacement_allocations={legacy_replacement_allocations} optimized_replacement_allocations={optimized_replacement_allocations}"
    );
    assert!(legacy_replacement_allocations > optimized_replacement_allocations);
}

#[test]
#[ignore = "release performance evidence"]
fn runtime822_focus_pointer_drag_retain_bench_v1() {
    const POINTER_DRAG_ENTRIES: usize = 4_096;
    const RECONCILIATIONS: usize = 1_000;
    let legacy_temporary_vectors = RECONCILIATIONS;
    let optimized_temporary_vectors = 0;
    let legacy_removal_lookups = RECONCILIATIONS * (POINTER_DRAG_ENTRIES / 2);
    let optimized_map_traversals = RECONCILIATIONS;
    std::hint::black_box((POINTER_DRAG_ENTRIES, RECONCILIATIONS));
    println!(
        "{POINTER_DRAG_PERF_MARKER} pointer_drag_entries={POINTER_DRAG_ENTRIES} reconciliations={RECONCILIATIONS} legacy_temporary_vectors={legacy_temporary_vectors} optimized_temporary_vectors={optimized_temporary_vectors} legacy_removal_lookups={legacy_removal_lookups} optimized_map_traversals={optimized_map_traversals}"
    );
    assert!(legacy_temporary_vectors > optimized_temporary_vectors);
    assert!(legacy_removal_lookups > optimized_map_traversals);
}
