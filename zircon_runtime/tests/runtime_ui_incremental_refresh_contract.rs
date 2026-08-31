#![cfg(feature = "ui")]

use std::sync::Arc;

use serde_json::json;
use zircon_runtime::ui::event_ui::UiEventManager;
use zircon_runtime::ui::surface::{
    hit_test_surface_frame, UiInvalidationReason, UiPropertyMutationRequest,
    UiPropertyMutationStatus, UiSurface,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{
        UiNodeDescriptor, UiNodeId, UiNodePath, UiNotification, UiPropertyDescriptor,
        UiReflectionNodePatch, UiReflectionSnapshot, UiStateFlags, UiTreeId, UiValueType,
    },
    layout::{
        Anchor, AxisConstraint, BoxConstraints, LayoutBoundary, Pivot, Position, StretchMode,
        UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiLayoutEngineBackend,
        UiLayoutEngineFallbackReason, UiLayoutEngineFamily, UiLayoutEngineRequest,
        UiLayoutEngineSelection, UiLayoutEngineSelectionReport, UiLayoutEngineSupport,
        UiLayoutEngineTaffyTreeBuildStats, UiPoint, UiSize, UiSlot, UiSlotKind,
    },
    pipeline::UiPipelineStage,
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn replacing_one_layout_selection_updates_aggregate_counts_in_place() {
    let replaced = UiLayoutEngineSelection {
        node_id: Some(UiNodeId::new(1)),
        request: UiLayoutEngineRequest::new(UiLayoutEngineFamily::Free),
        requested_backend: UiLayoutEngineBackend::Zircon,
        selected_backend: UiLayoutEngineBackend::Zircon,
        support: UiLayoutEngineSupport::Native,
        fallback_reason: None,
        taffy_tree_build: None,
    };
    let untouched = UiLayoutEngineSelection {
        node_id: Some(UiNodeId::new(2)),
        request: UiLayoutEngineRequest::new(UiLayoutEngineFamily::Grid),
        requested_backend: UiLayoutEngineBackend::Taffy,
        selected_backend: UiLayoutEngineBackend::Zircon,
        support: UiLayoutEngineSupport::Fallback,
        fallback_reason: Some(UiLayoutEngineFallbackReason::UnsupportedFamily),
        taffy_tree_build: None,
    };
    let replacement = UiLayoutEngineSelection {
        node_id: Some(UiNodeId::new(1)),
        request: UiLayoutEngineRequest::new(UiLayoutEngineFamily::Grid),
        requested_backend: UiLayoutEngineBackend::Taffy,
        selected_backend: UiLayoutEngineBackend::Taffy,
        support: UiLayoutEngineSupport::Unsupported,
        fallback_reason: Some(UiLayoutEngineFallbackReason::TaffyTreeBuildFailed),
        taffy_tree_build: Some(UiLayoutEngineTaffyTreeBuildStats {
            build_count: 2,
            node_count: 9,
        }),
    };
    let mut report =
        UiLayoutEngineSelectionReport::from_selections(vec![replaced, untouched.clone()]);
    let expected =
        UiLayoutEngineSelectionReport::from_selections(vec![replacement.clone(), untouched]);

    assert!(report.replace_selection_at(0, replacement));
    assert_eq!(report, expected);
}

#[test]
fn reflection_patch_batch_is_atomic_and_broadcasts_one_diff_per_tree() {
    let mut manager = UiEventManager::default();
    let node_path = UiNodePath::new("editor/workbench/scene");
    manager.replace_tree(UiReflectionSnapshot::new(
        UiTreeId::new("editor.workbench"),
        vec![UiNodeId::new(1)],
        vec![
            UiNodeDescriptor::new(UiNodeId::new(1), node_path.clone(), "SceneView", "Scene")
                .with_property(UiPropertyDescriptor::new(
                    "transient.hovered",
                    UiValueType::Bool,
                    json!(false),
                )),
        ],
    ));
    let (_subscription_id, receiver) = manager.subscribe();

    let diffs = manager
        .apply_reflection_patches(&[
            UiReflectionNodePatch::new(node_path.clone())
                .with_property("transient.hovered", json!(true)),
            UiReflectionNodePatch::new(node_path.clone()).with_pressed(true),
        ])
        .expect("validated patches should apply atomically");

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].changed_nodes, vec![UiNodeId::new(1)]);
    let node = manager.query_node(&node_path).expect("patched node");
    assert_eq!(
        node.properties["transient.hovered"].reflected_value,
        json!(true)
    );
    assert!(node.state_flags.pressed);
    assert!(matches!(
        receiver.recv().unwrap(),
        UiNotification::ReflectionDiff(diff) if diff.changed_nodes == vec![UiNodeId::new(1)]
    ));

    let error = manager
        .apply_reflection_patches(&[
            UiReflectionNodePatch::new(node_path.clone())
                .with_property("transient.hovered", json!(false)),
            UiReflectionNodePatch::new(node_path.clone())
                .with_property("missing.property", json!(true)),
        ])
        .expect_err("one invalid patch must reject the whole batch");
    assert!(matches!(
        error,
        zircon_runtime_interface::ui::event_ui::UiInvocationError::UnknownProperty { .. }
    ));
    assert_eq!(
        manager
            .query_property(&node_path, "transient.hovered")
            .expect("existing reflected property")
            .reflected_value,
        json!(true),
        "validation failure must not leave the valid prefix applied"
    );
}

#[test]
fn single_node_layout_dirty_patches_only_changed_post_layout_state() {
    let mut surface = flat_surface(2);
    resize_last_child_and_invalidate_layout(&mut surface, 2);

    let report = surface.rebuild_dirty(root_size()).unwrap();
    let pipeline = report.pipeline_report(1);

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(
        surface.last_layout_geometry_changed_node_ids(),
        &std::collections::BTreeSet::from([child_id(1)])
    );
    assert_eq!(report.arranged_outer_node_visit_count, 1, "{report:#?}");
    assert_eq!(report.hit_grid_outer_node_visit_count, 1, "{report:#?}");
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::PostLayout)
            .unwrap()
            .counters
            .post_layout_outer_node_visit_count,
        1
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::Picking)
            .unwrap()
            .counters
            .picking_outer_node_visit_count,
        1
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .counters
            .render_extract_outer_node_visit_count,
        1
    );
}

#[test]
fn direct_node_map_mutation_is_included_in_the_next_incremental_rebuild() {
    let mut surface = flat_surface(2);
    let changed_node_id = child_id(1);
    let changed = surface
        .tree
        .nodes
        .get_mut(&changed_node_id)
        .expect("changed node should exist");
    changed.constraints.width = fixed_constraint(20.0);
    changed.dirty.layout = true;
    changed.dirty.hit_test = true;
    changed.dirty.render = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(
        surface.last_layout_geometry_changed_node_ids(),
        &std::collections::BTreeSet::from([changed_node_id])
    );
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 1);
    assert_eq!(report.render_outer_node_visit_count, 1);
}

#[test]
fn surface_report_exposes_text_cache_hits_and_misses() {
    let mut surface = text_surface();
    surface
        .tree
        .node_mut(child_id(0))
        .expect("text node should exist")
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(
            r#"
text = "Measured label"
editable_text = true
font_size = 10.0
line_height = 12.0
wrap = "Word"
"#,
        )
        .expect("text metadata should parse"),
        ..UiTemplateNodeMetadata::default()
    });

    surface.compute_layout(root_size()).unwrap();
    let first = surface.surface_frame().pipeline_report.clone();
    let first_text = first
        .stage_report(UiPipelineStage::TextMeasure)
        .expect("text stage should be reported");

    assert!(!first_text.skipped);
    assert_eq!(first_text.counters.text_measure_cache_miss_count, 1);
    assert_eq!(
        first_text.counters.text_layout_cache_miss_count, 1,
        "first text counters={:?}, render_commands={:?}",
        first_text.counters, surface.render_extract.list.commands
    );
    assert!(first_text.counters.text_shape_cache_miss_count > 0);

    surface.rebuild();
    let forced_rebuild = surface.surface_frame().pipeline_report.clone();
    let forced_rebuild_text = forced_rebuild
        .stage_report(UiPipelineStage::TextMeasure)
        .expect("forced-rebuild text stage should be reported");

    assert!(!forced_rebuild_text.skipped);
    assert_eq!(forced_rebuild_text.counters.text_layout_cache_hit_count, 1);
    assert_eq!(forced_rebuild_text.counters.text_layout_cache_miss_count, 0);
    assert_eq!(forced_rebuild_text.counters.text_shape_cache_miss_count, 0);
}

#[test]
fn fixed_size_text_dirty_reextracts_only_the_changed_node() {
    let mut surface = text_surface();
    surface
        .tree
        .node_mut(child_id(0))
        .expect("text node should exist")
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(
            r#"
text = "Before"
font_size = 10.0
line_height = 12.0
"#,
        )
        .expect("text metadata should parse"),
        ..UiTemplateNodeMetadata::default()
    });
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
        .tree
        .node_mut(child_id(0))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("text metadata should exist")
        .attributes
        .insert("text".to_string(), toml::Value::String("After".to_string()));
    surface
        .invalidate_node(child_id(0), UiInvalidationReason::Text)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 0);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert!(surface
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == child_id(0) && command.text.as_deref() == Some("After")));
}

#[test]
fn single_node_resource_dirty_reextracts_only_the_changed_node() {
    let mut surface = flat_surface(2);
    let changed_node_id = child_id(0);
    let initial_command_count = surface.render_extract.list.commands.len();
    surface
        .tree
        .node_mut(changed_node_id)
        .and_then(|node| node.template_metadata.as_mut())
        .expect("button metadata should exist")
        .style_overrides
        .insert(
            "background_color".to_string(),
            toml::Value::String("#ff0000".to_string()),
        );
    surface
        .invalidate_node(changed_node_id, UiInvalidationReason::Resource)
        .expect("changed button should invalidate");

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 0);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert_eq!(
        surface.render_extract.list.commands.len(),
        initial_command_count
    );
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.node_id == changed_node_id
            && command.style.background_color.as_deref() == Some("#ff0000")
    }));
}

#[test]
fn single_node_render_only_style_mutation_reextracts_only_the_changed_node() {
    let mut surface = flat_surface(2);
    let changed_node_id = child_id(0);
    let initial_command_count = surface.render_extract.list.commands.len();
    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            changed_node_id,
            "background_color",
            UiValue::String("#00ff00".to_string()),
        ))
        .expect("button color mutation should succeed");

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            render: true,
            ..UiDirtyFlags::default()
        }
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_eq!(report.layout_visited_node_count, 0);
    assert_eq!(report.arranged_outer_node_visit_count, 0);
    assert_eq!(report.hit_grid_outer_node_visit_count, 0);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert_eq!(
        surface.render_extract.list.commands.len(),
        initial_command_count
    );
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.node_id == changed_node_id
            && command.style.background_color.as_deref() == Some("#00ff00")
    }));
}

#[test]
fn deserialized_surface_rebuilds_cached_geometry_for_the_first_root_size() {
    let old_root_size = UiSize::new(100.0, 40.0);
    let new_root_size = UiSize::new(240.0, 90.0);
    let root_id = UiNodeId::new(1);
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.incremental_refresh.deserialize_root_size",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_container(UiContainerKind::Free)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface.compute_layout(old_root_size).unwrap();
    surface.clear_dirty_flags();
    let old_frame = surface
        .arranged_tree
        .get(root_id)
        .expect("root should be arranged")
        .frame;
    assert_eq!(
        UiSize::new(old_frame.width, old_frame.height),
        old_root_size
    );

    let serialized = serde_json::to_string(&surface).unwrap();
    let mut restored: UiSurface = serde_json::from_str(&serialized).unwrap();
    let report = restored.rebuild_dirty(new_root_size).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(report.layout_visited_node_count, 1);
    let new_frame = restored
        .arranged_tree
        .get(root_id)
        .expect("deserialized root should be rearranged")
        .frame;
    assert_eq!(
        UiSize::new(new_frame.width, new_frame.height),
        new_root_size
    );
}

#[test]
fn same_cardinality_slot_replacement_recomputes_responsive_grid_placement() {
    let root_id = UiNodeId::new(1);
    let old_child_id = UiNodeId::new(2);
    let new_child_id = UiNodeId::new(3);
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.incremental_refresh.slot_replacement",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            })
            .with_container(UiContainerKind::GridBox(Default::default()))
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Grid".to_string(),
                attributes: toml::from_str(
                    r#"
container = true
columns = 12
"#,
                )
                .expect("grid container metadata should parse"),
                ..UiTemplateNodeMetadata::default()
            })
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(root_id, responsive_grid_child(old_child_id, "root/old", 3))
        .unwrap();
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id, old_child_id, UiSlotKind::Free));
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    assert_eq!(surface.tree.layout_slots()[0].kind, UiSlotKind::Grid);
    assert_eq!(
        surface.tree.layout_slots()[0]
            .grid_placement
            .expect("initial responsive placement should exist")
            .column_span,
        3
    );

    surface.detach_subtree_to_pool(old_child_id).unwrap();
    surface
        .tree
        .insert_child(root_id, responsive_grid_child(new_child_id, "root/new", 6))
        .unwrap();
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id, new_child_id, UiSlotKind::Free));

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(surface.tree.layout_slots().len(), 1);
    assert_eq!(surface.tree.layout_slots()[0].child_id, new_child_id);
    assert_eq!(surface.tree.layout_slots()[0].kind, UiSlotKind::Grid);
    assert_eq!(
        surface.tree.layout_slots()[0]
            .grid_placement
            .expect("replacement responsive placement should exist")
            .column_span,
        6
    );
}

#[test]
fn pointer_node_crossing_hit_grid_cells_is_indexed_once() {
    let root_id = UiNodeId::new(1);
    let pointer_id = UiNodeId::new(2);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.incremental_layout.cells"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            })
            .with_container(UiContainerKind::Free)
            .with_layout_boundary(LayoutBoundary::ParentDirected)
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                hoverable: true,
                ..Default::default()
            }),
    );
    surface
        .tree
        .insert_child(
            root_id,
            UiTreeNode::new(pointer_id, UiNodePath::new("root/pointer"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(20.0),
                    height: fixed_constraint(20.0),
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    hoverable: true,
                    ..Default::default()
                })
                .with_layout_boundary(LayoutBoundary::ParentDirected),
        )
        .expect("pointer node should insert");
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id, pointer_id, UiSlotKind::Free));
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();

    surface
        .set_free_slot_canvas_placement(
            root_id,
            pointer_id,
            UiCanvasSlotPlacement::new(
                Anchor::new(0.0, 0.0),
                Pivot::new(0.0, 0.0),
                Position::new(72.0, 0.0),
            ),
        )
        .expect("pointer placement should change");

    let report = surface.rebuild_dirty(root_size()).unwrap();
    let hit = surface.hit_test(UiPoint::new(76.0, 8.0));

    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 1);
    assert_eq!(hit.top_hit, Some(pointer_id));
    assert_eq!(
        hit.stacked
            .iter()
            .filter(|node_id| **node_id == pointer_id)
            .count(),
        1
    );
}

#[test]
fn surface_frame_publication_reuses_stable_arc_and_preserves_old_generation() {
    let mut surface = flat_surface(2);

    let first = surface.surface_frame();
    let stable = surface.surface_frame();

    assert!(Arc::ptr_eq(&first, &stable));
    assert_eq!(first.generation, stable.generation);
    assert!(first.render_extract.list.commands.iter().all(|command| {
        command.node_id != child_id(0)
            || command.style.background_color.as_deref() != Some("#00ff00")
    }));

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            child_id(0),
            "background_color",
            UiValue::String("#00ff00".to_string()),
        ))
        .expect("button color mutation should succeed");
    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    surface.rebuild_dirty(root_size()).unwrap();

    let next = surface.surface_frame();
    let stable_next = surface.surface_frame();

    assert!(!Arc::ptr_eq(&first, &next));
    assert!(next.generation > first.generation);
    assert!(Arc::ptr_eq(&next, &stable_next));
    assert!(next.render_extract.list.commands.iter().any(|command| {
        command.node_id == child_id(0)
            && command.style.background_color.as_deref() == Some("#00ff00")
    }));
    assert!(first.render_extract.list.commands.iter().all(|command| {
        command.node_id != child_id(0)
            || command.style.background_color.as_deref() != Some("#00ff00")
    }));
}

#[test]
fn surface_frame_hit_test_uses_shared_route_authority_without_arranged_lookups() {
    let mut surface = pointer_surface("runtime.ui.cached_hit");
    surface.rebuild();

    let mut frame = surface.surface_frame();
    let entry = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == child_id(0))
        .expect("button should be cached in the hit grid");
    let route = frame
        .hit_grid
        .route_nodes
        .get(entry.route_node_index as usize)
        .expect("button should reference the shared route authority");
    assert_eq!(route.effective_input_policy, UiInputPolicy::Receive);
    assert_eq!(route.node_id, child_id(0));

    Arc::make_mut(&mut Arc::make_mut(&mut frame).arranged_tree)
        .nodes
        .clear();
    let hit = hit_test_surface_frame(&frame, UiPoint::new(20.0, 20.0));

    assert_eq!(hit.top_hit, Some(child_id(0)));
    assert_eq!(hit.stacked, vec![child_id(0)]);
    assert_eq!(hit.path.root_to_leaf, vec![UiNodeId::new(1), child_id(0)]);
    assert_eq!(
        hit.path.bubble_route().collect::<Vec<_>>(),
        vec![child_id(0), UiNodeId::new(1)]
    );
    assert_eq!(
        hit.top_entry(&frame.hit_grid)
            .and_then(|entry| entry.control_id.as_deref()),
        Some("cached.button")
    );
}

#[test]
fn hit_grid_without_route_authority_is_rejected() {
    let mut surface = pointer_surface("runtime.ui.legacy_hit");
    surface.rebuild();

    let frame = surface.surface_frame();
    let mut serialized = serde_json::to_value(&frame).expect("surface frame should serialize");
    serialized["hit_grid"]
        .as_object_mut()
        .expect("hit grid should serialize as an object")
        .remove("route_nodes");

    assert!(
        serde_json::from_value::<zircon_runtime_interface::ui::surface::UiSurfaceFrame>(serialized)
            .is_err()
    );
}

fn pointer_surface(tree_id: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(tree_id));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 90.0))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(child_id(0), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(12.0, 12.0, 80.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_template_metadata(UiTemplateNodeMetadata {
                    control_id: Some("cached.button".to_string()),
                    ..UiTemplateNodeMetadata::default()
                })
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    ..UiStateFlags::default()
                }),
        )
        .expect("button should insert");
    surface
}

fn text_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.incremental_refresh.text"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0))
            .with_container(UiContainerKind::Free)
            .with_layout_boundary(LayoutBoundary::ParentDirected)
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                ..UiStateFlags::default()
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(child_id(0), UiNodePath::new("root/text"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .expect("text node should insert");
    surface
}

fn responsive_grid_child(node_id: UiNodeId, path: &str, span: usize) -> UiTreeNode {
    UiTreeNode::new(node_id, UiNodePath::new(path))
        .with_constraints(BoxConstraints {
            width: fixed_constraint(40.0),
            height: fixed_constraint(20.0),
        })
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "Grid".to_string(),
            attributes: toml::from_str(&format!("size = {span}"))
                .expect("grid item metadata should parse"),
            ..UiTemplateNodeMetadata::default()
        })
        .with_input_policy(UiInputPolicy::Ignore)
}

#[test]
#[ignore = "explicit M4/M5 scale matrix; run at milestone performance gates"]
fn stable_and_single_node_dirty_scale_matrix_keeps_local_work_constant() {
    for child_count in [1_usize, 100, 1_000, 10_000] {
        let mut surface = flat_surface(child_count);
        let stable = surface.rebuild_dirty(root_size()).unwrap();
        assert!(!stable.dirty_flags.any(), "{child_count}");
        assert!(!stable.layout_recomputed, "{child_count}");
        assert!(!stable.arranged_rebuilt, "{child_count}");
        assert!(!stable.hit_grid_rebuilt, "{child_count}");
        assert!(!stable.render_rebuilt, "{child_count}");
        assert_eq!(stable.layout_visited_node_count, 0, "{child_count}");
        assert_eq!(stable.arranged_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.hit_grid_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.render_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.text_measure_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_measure_cache_miss_count, 0, "{child_count}");
        assert_eq!(stable.text_layout_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_layout_cache_miss_count, 0, "{child_count}");
        assert_eq!(stable.text_shape_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_shape_cache_miss_count, 0, "{child_count}");

        resize_last_child_and_invalidate_layout(&mut surface, child_count);

        let report = surface.rebuild_dirty(root_size()).unwrap();

        assert_eq!(report.layout_visited_node_count, 1, "{child_count}");
        assert_eq!(report.arranged_outer_node_visit_count, 1, "{child_count}");
        assert_eq!(report.hit_grid_outer_node_visit_count, 1, "{child_count}");
        assert_eq!(report.render_outer_node_visit_count, 1, "{child_count}");

        mutate_last_child_render_only_style(&mut surface, child_count);

        let style_report = surface.rebuild_dirty(root_size()).unwrap();

        assert_eq!(style_report.layout_visited_node_count, 0, "{child_count}");
        assert_eq!(
            style_report.arranged_outer_node_visit_count, 0,
            "{child_count}"
        );
        assert_eq!(
            style_report.hit_grid_outer_node_visit_count, 0,
            "{child_count}"
        );
        assert_eq!(
            style_report.render_outer_node_visit_count, 1,
            "{child_count}"
        );
        eprintln!(
            "nodes={} layout_visited={} arranged_outer_visited={} hit_outer_visited={} render_outer_visited={} layout_us={} arranged_us={} hit_us={} render_us={} style_layout_visited={} style_arranged_outer_visited={} style_hit_outer_visited={} style_render_outer_visited={} style_layout_us={} style_arranged_us={} style_hit_us={} style_render_us={}",
            child_count + 1,
            report.layout_visited_node_count,
            report.arranged_outer_node_visit_count,
            report.hit_grid_outer_node_visit_count,
            report.render_outer_node_visit_count,
            report.layout_elapsed_micros,
            report.arranged_elapsed_micros,
            report.hit_grid_elapsed_micros,
            report.render_elapsed_micros,
            style_report.layout_visited_node_count,
            style_report.arranged_outer_node_visit_count,
            style_report.hit_grid_outer_node_visit_count,
            style_report.render_outer_node_visit_count,
            style_report.layout_elapsed_micros,
            style_report.arranged_elapsed_micros,
            style_report.hit_grid_elapsed_micros,
            style_report.render_elapsed_micros,
        );
    }
}

fn flat_surface(child_count: usize) -> UiSurface {
    let mut surface = unbuilt_flat_surface(child_count);
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn unbuilt_flat_surface(child_count: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.incremental_refresh.scale.{child_count}"
    )));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            })
            .with_container(UiContainerKind::Free)
            .with_layout_boundary(LayoutBoundary::ParentDirected)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    for child_index in 0..child_count {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    child_id(child_index),
                    UiNodePath::new(format!("root/{child_index}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                })
                .with_layout_boundary(LayoutBoundary::ParentDirected)
                .with_input_policy(UiInputPolicy::Receive)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some(format!("scale.button.{child_index}")),
                    ..UiTemplateNodeMetadata::default()
                })
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    ..UiStateFlags::default()
                }),
            )
            .expect("scale-matrix node should insert");
    }
    surface
}

fn resize_last_child_and_invalidate_layout(surface: &mut UiSurface, child_count: usize) {
    let changed_node_id = child_id(child_count - 1);
    surface
        .tree
        .node_mut(changed_node_id)
        .expect("changed node should exist")
        .constraints
        .width = fixed_constraint(20.0);
    surface
        .invalidate_node(changed_node_id, UiInvalidationReason::Layout)
        .expect("changed node should invalidate");
}

fn mutate_last_child_render_only_style(surface: &mut UiSurface, child_count: usize) {
    let changed_node_id = child_id(child_count - 1);
    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            changed_node_id,
            "background_color",
            UiValue::String("#00ff00".to_string()),
        ))
        .expect("changed button style should mutate");
    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            render: true,
            ..UiDirtyFlags::default()
        }
    );
}

fn child_id(child_index: usize) -> UiNodeId {
    UiNodeId::new(child_index as u64 + 2)
}

fn fixed_constraint(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: value,
        preferred: value,
        max: value,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn root_size() -> UiSize {
    UiSize::new(120.0, 60.0)
}
