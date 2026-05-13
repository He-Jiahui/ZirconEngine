use crate::ui::{
    surface::{
        UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface, UiSurfaceRebuildReport,
    },
    tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeLayoutExt},
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiDispatchEffect, UiDispatchReply, UiInputEvent, UiKeyboardInputEvent,
        UiKeyboardInputState, UiRedrawRequestReason,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, LayoutBoundary, StretchMode, UiContainerKind, UiSize,
    },
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
};

#[test]
fn surface_dirty_rebuild_separates_hit_input_render_and_legacy_state_flags() {
    let mut surface = test_surface();

    mark_structured_dirty(
        &mut surface,
        UiDirtyFlags {
            hit_test: true,
            ..Default::default()
        },
    );
    let hit_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        hit_report,
        UiDirtyFlags {
            hit_test: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: true,
            hit_grid: true,
            render: false,
        },
    );
    assert_dirty_cleared(&surface);

    mark_structured_dirty(
        &mut surface,
        UiDirtyFlags {
            input: true,
            ..Default::default()
        },
    );
    let input_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        input_report,
        UiDirtyFlags {
            input: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: true,
            hit_grid: true,
            render: false,
        },
    );
    assert_dirty_cleared(&surface);

    mark_structured_dirty(
        &mut surface,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
    );
    let render_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        render_report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);

    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .state_flags
        .dirty = true;
    let legacy_report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        legacy_report,
        UiDirtyFlags {
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: true,
            hit_grid: true,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_rebuild_recomputes_layout_for_structural_domains() {
    for dirty_flags in [
        UiDirtyFlags {
            layout: true,
            ..Default::default()
        },
        UiDirtyFlags {
            style: true,
            ..Default::default()
        },
        UiDirtyFlags {
            text: true,
            ..Default::default()
        },
        UiDirtyFlags {
            visible_range: true,
            ..Default::default()
        },
    ] {
        let mut surface = test_surface();

        mark_structured_dirty(&mut surface, dirty_flags);
        let report = surface.rebuild_dirty(root_size()).unwrap();

        assert_report_phases(
            &surface,
            report,
            dirty_flags,
            ExpectedPhases {
                layout: true,
                arranged: true,
                hit_grid: true,
                render: true,
            },
        );
        assert_dirty_cleared(&surface);
    }
}

#[test]
fn surface_dirty_layout_skips_siblings_under_non_auto_parent() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    let sibling_frame = surface
        .arranged_tree
        .get(sibling_id())
        .expect("sibling should be arranged")
        .frame;

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .width = fixed_constraint(60.0);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .dirty
        .layout = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.layout_skipped_node_count, 2);
    assert_eq!(report.layout_geometry_changed_node_count, 1);
    assert_eq!(report.render_command_rebuilt_count, 1);
    assert_eq!(report.render_damage_rect_count, 1);
    assert_eq!(
        surface
            .arranged_tree
            .get(sibling_id())
            .expect("sibling should stay arranged")
            .frame,
        sibling_frame
    );
    assert_dirty_cleared_for(&surface, primary_id());
}

#[test]
fn surface_dirty_layout_revisits_auto_parent_when_child_size_changes() {
    let mut surface = sibling_surface(
        UiContainerKind::VerticalBox(Default::default()),
        LayoutBoundary::ParentDirected,
    );

    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .constraints
        .height = fixed_constraint(40.0);
    surface
        .tree
        .node_mut(primary_id())
        .expect("primary node should exist")
        .dirty
        .layout = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 3);
    assert_eq!(report.layout_skipped_node_count, 0);
    assert_eq!(report.layout_geometry_changed_node_count, 2);
    assert_eq!(
        surface
            .arranged_tree
            .get(sibling_id())
            .expect("auto-layout sibling should be rearranged")
            .frame
            .y,
        40.0
    );
    assert_dirty_cleared_for(&surface, primary_id());
}

#[test]
fn surface_dirty_render_reuses_unchanged_commands_without_damage() {
    let mut surface = test_surface();
    let command_count = surface.render_extract.list.commands.len();

    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .dirty
        .render = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.render_rebuilt);
    assert_eq!(report.render_command_reused_count, command_count);
    assert_eq!(report.render_command_rebuilt_count, 0);
    assert_eq!(report.render_damage_rect_count, 0);
    assert!(surface
        .render_extract
        .list
        .to_paint_elements()
        .iter()
        .all(|element| element.cache_generation.is_some()));
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_render_only_metadata_does_not_trigger_hit_or_input_rebuild() {
    let mut surface = test_surface();
    let metadata = UiTemplateNodeMetadata {
        component: "Button".to_string(),
        control_id: Some("DirtyDomainButton".to_string()),
        attributes: toml::from_str("material_tone = 'primary'").unwrap(),
        ..Default::default()
    };
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .template_metadata = Some(metadata);
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "material_tone",
            UiValue::String("secondary".to_string()),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert!(
        !surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_text_edit_visual_metadata_stays_render_only() {
    let mut surface = test_surface();
    let metadata = UiTemplateNodeMetadata {
        component: "TextField".to_string(),
        control_id: Some("DirtyDomainTextField".to_string()),
        attributes: toml::from_str(
            r#"
value = "Runtime"
caret_offset = 3
selection_anchor = 3
selection_focus = 3
composition_text = ""
"#,
        )
        .unwrap(),
        ..Default::default()
    };
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .template_metadata = Some(metadata);
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "caret_offset",
            UiValue::Int(4),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_render_only_dispatch_effect_does_not_trigger_hit_or_input_rebuild() {
    let mut surface = test_surface();
    surface.clear_dirty_flags();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::DirtyRedraw {
            target: button_id(),
            dirty: UiDirtyFlags {
                render: true,
                ..Default::default()
            },
            reason: UiRedrawRequestReason::Style,
        }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert!(
        !surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_route_state_mutations_keep_legacy_dirty_for_bridges() {
    let mut surface = test_surface();

    let visibility = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "visibility",
            UiValue::Enum("hidden".to_string()),
        ))
        .unwrap();

    assert_eq!(visibility.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .visibility,
        UiVisibility::Hidden
    );
    assert!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        }
    );

    surface.clear_dirty_flags();
    let input_policy = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "input_policy",
            UiValue::Enum("ignore".to_string()),
        ))
        .unwrap();

    assert_eq!(input_policy.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .input_policy,
        UiInputPolicy::Ignore
    );
    assert!(
        surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        }
    );
}

#[test]
fn surface_dirty_layout_marking_keeps_structured_domains_precise() {
    let mut surface = test_surface();

    surface.tree.mark_layout_dirty(button_id()).unwrap();

    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..Default::default()
        }
    );
    assert!(
        !surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );
    assert!(
        !surface
            .tree
            .node(root_id())
            .expect("root node should exist")
            .state_flags
            .dirty
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
