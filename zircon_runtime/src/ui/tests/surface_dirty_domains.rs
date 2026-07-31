use crate::ui::{
    surface::{
        UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface, UiSurfaceRebuildReport,
    },
    tree::UiRuntimeTreeLayoutExt,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiDispatchEffect, UiDispatchReply, UiInputEvent, UiKeyboardInputEvent,
        UiKeyboardInputState, UiRedrawRequestReason,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, LayoutBoundary, StretchMode, UiContainerKind,
        UiLayoutEngineBackend, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
        UiLayoutEngineSelectionReport, UiLayoutEngineSupport, UiSize,
    },
    surface::{UiSurfaceDebugOptions, UiSurfaceDebugSnapshot},
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
};

mod incremental_layout;
mod mutation_state;
mod rebuild_domains;
mod render_domains;

#[test]
fn surface_rebuild_collects_dirty_flags_and_node_count_in_one_tree_pass() {
    let source = include_str!("../surface/surface/rebuild.rs");

    assert!(
        !source.contains(
            "let dirty_flags = self.dirty_flags();\n        let dirty_node_count = dirty_node_count"
        ),
        "surface rebuild must not scan the full tree separately for dirty flags and node count"
    );
    assert!(
        !source.contains(
            "let dirty = self.dirty_flags();\n        let dirty_node_count = dirty_node_count"
        ),
        "incremental rebuild must collect both dirty summaries in one pass"
    );
}

fn test_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.dirty_domains"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root")).with_constraints(BoxConstraints {
            width: fixed_constraint(120.0),
            height: fixed_constraint(60.0),
        }),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(button_id(), UiNodePath::new("root/button"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(80.0),
                    height: fixed_constraint(24.0),
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn mark_structured_dirty(surface: &mut UiSurface, dirty_flags: UiDirtyFlags) {
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .dirty = dirty_flags;
}

fn assert_report_phases(
    surface: &UiSurface,
    report: UiSurfaceRebuildReport,
    expected_dirty: UiDirtyFlags,
    expected_phases: ExpectedPhases,
) {
    assert_eq!(report.dirty_flags, expected_dirty);
    assert_eq!(report.dirty_node_count, 1);
    assert_eq!(report.layout_recomputed, expected_phases.layout);
    assert_eq!(report.arranged_rebuilt, expected_phases.arranged);
    assert_eq!(report.hit_grid_rebuilt, expected_phases.hit_grid);
    assert_eq!(report.render_rebuilt, expected_phases.render);
    assert_eq!(
        report.arranged_node_count,
        surface.arranged_tree.nodes.len()
    );
    assert_eq!(
        report.render_command_count,
        surface.render_extract.list.commands.len()
    );
    assert_eq!(
        report.hit_grid_entry_count,
        surface.hit_test.grid.entries.len()
    );
    assert_eq!(
        report.hit_grid_cell_count,
        surface.hit_test.grid.cells.len()
    );
    assert_eq!(surface.surface_frame().last_rebuild, report.debug_stats());
}

fn assert_dirty_cleared(surface: &UiSurface) {
    assert!(!surface.dirty_flags().any());
    assert_dirty_cleared_for(surface, button_id());
}

fn assert_dirty_cleared_for(surface: &UiSurface, node_id: UiNodeId) {
    assert!(
        !surface
            .tree
            .node(node_id)
            .expect("node should exist")
            .state_flags
            .dirty
    );
}

fn sibling_surface(container: UiContainerKind, boundary: LayoutBoundary) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.incremental_layout"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            })
            .with_container(container)
            .with_layout_boundary(boundary),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(primary_id(), UiNodePath::new("root/primary")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(sibling_id(), UiNodePath::new("root/sibling")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn layout_route_merge_surface() -> UiSurface {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .container = UiContainerKind::HorizontalBox(Default::default());
    surface
        .tree
        .insert_child(
            primary_id(),
            UiTreeNode::new(grandchild_id(), UiNodePath::new("root/primary/leaf"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(16.0),
                    height: fixed_constraint(16.0),
                }),
        )
        .unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn assert_route(
    report: &UiLayoutEngineSelectionReport,
    node_id: UiNodeId,
    family: UiLayoutEngineFamily,
    backend: UiLayoutEngineBackend,
    support: UiLayoutEngineSupport,
    fallback_reason: Option<UiLayoutEngineFallbackReason>,
) {
    let selection = report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(node_id))
        .unwrap_or_else(|| panic!("route for node {node_id:?} should be reported"));
    assert_eq!(selection.request.family, family, "{report:#?}");
    assert_eq!(selection.selected_backend, backend, "{report:#?}");
    assert_eq!(selection.support, support, "{report:#?}");
    assert_eq!(selection.fallback_reason, fallback_reason, "{report:#?}");
}

fn route_count_for_node(report: &UiLayoutEngineSelectionReport, node_id: UiNodeId) -> usize {
    report
        .selections
        .iter()
        .filter(|selection| selection.node_id == Some(node_id))
        .count()
}

fn assert_fallback_reason_count(
    report: &UiLayoutEngineSelectionReport,
    reason: UiLayoutEngineFallbackReason,
    count: u64,
) {
    assert_eq!(
        report
            .fallback_reason_counts
            .iter()
            .find(|reason_count| reason_count.reason == Some(reason))
            .map(|reason_count| reason_count.count),
        Some(count),
        "{report:#?}"
    );
}

fn assert_layout_engine_report_exported(
    surface: &UiSurface,
    expected: &UiLayoutEngineSelectionReport,
) {
    assert_eq!(&surface.surface_frame().layout_engine_report, expected);
    assert_eq!(&surface.debug_snapshot().layout_engine_report, expected);

    let snapshot_json = surface
        .debug_snapshot_json(&UiSurfaceDebugOptions::default())
        .expect("incremental layout debug snapshot should serialize");
    let snapshot: UiSurfaceDebugSnapshot =
        serde_json::from_str(&snapshot_json).expect("incremental layout debug snapshot JSON");
    assert_eq!(&snapshot.layout_engine_report, expected);
}

fn root_id() -> UiNodeId {
    UiNodeId::new(1)
}

fn button_id() -> UiNodeId {
    UiNodeId::new(2)
}

fn primary_id() -> UiNodeId {
    UiNodeId::new(2)
}

fn sibling_id() -> UiNodeId {
    UiNodeId::new(3)
}

fn grandchild_id() -> UiNodeId {
    UiNodeId::new(4)
}

fn root_size() -> UiSize {
    UiSize::new(120.0, 60.0)
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: Default::default(),
        state: UiKeyboardInputState::Pressed,
        key_code: 13,
        scan_code: None,
        physical_key: "Enter".to_string(),
        logical_key: "Enter".to_string(),
        text: None,
    })
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

#[derive(Clone, Copy)]
struct ExpectedPhases {
    layout: bool,
    arranged: bool,
    hit_grid: bool,
    render: bool,
}
