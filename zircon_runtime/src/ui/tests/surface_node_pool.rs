use std::collections::BTreeMap;

use crate::ui::{
    dispatch::UiPointerDispatcher,
    surface::{UiSurface, UiSurfaceNodePool},
};
use zircon_runtime_interface::ui::{
    dispatch::{UiDragSessionId, UiPointerEvent, UiPointerId},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, StretchMode, UiPoint, UiSize},
    surface::{UiPointerButton, UiPointerEventKind},
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn surface_node_pool_detach_filters_slots_once_for_the_entire_subtree() {
    let source = include_str!("../surface/node_pool.rs");
    let retain = source
        .find("tree.retain_layout_slots")
        .expect("node-pool detach should remove detached slots");
    let recycle_loop = source
        .find("for node_id in node_ids.iter().copied().rev()")
        .expect("node-pool detach should recycle nodes bottom-up");

    assert!(
        retain < recycle_loop,
        "slot filtering must run once before the detached-node recycle loop"
    );
}

#[test]
fn surface_node_pool_reinsert_uses_the_ui_tree_paint_order_authority() {
    let source = include_str!("../surface/node_pool.rs");
    let start = source
        .find("pub(crate) fn insert_or_reuse_pooled_child(")
        .expect("node-pool child insertion entry point");
    let end = source[start..]
        .find("fn collect_subtree_node_ids(")
        .map(|offset| start + offset)
        .expect("node-pool child insertion boundary");
    let insertion = &source[start..end];

    assert!(insertion.contains("tree.insert_child(parent_id, node)?;"));
    assert!(!insertion.contains("next_paint_order"));
}

#[test]
fn surface_node_pool_reuses_detached_template_node_and_resets_transient_state() {
    let mut surface = pooled_surface();
    surface.focus.focused = Some(child_id());
    surface.focus.captured = Some(child_id());
    surface.focus.pressed = Some(child_id());
    surface.focus.hovered = vec![child_id(), root_id()];
    surface
        .input
        .set_pointer_capture_for_id(UiPointerId::new(7), child_id());
    surface.input.high_precision_owner = Some(child_id());
    surface.input.input_method_owner = Some(child_id());
    surface
        .input
        .begin_drag_drop(
            child_id(),
            child_id(),
            UiPointerId::new(7),
            Some(UiDragSessionId::new(70)),
            Some(UiPoint::new(12.0, 12.0)),
            None,
        )
        .unwrap();
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(12.0, 12.0)),
        )
        .unwrap();
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(12.0, 12.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert!(
        surface
            .component_state(child_id())
            .expect("component state should exist before detach")
            .flags
            .pressed
    );
    surface.focus.hovered = vec![child_id(), root_id()];
    let child = surface
        .tree
        .node_mut(child_id())
        .expect("child should exist before detach");
    child.state_flags.pressed = true;
    child.state_flags.dirty = true;
    child.dirty = UiDirtyFlags {
        style: true,
        text: true,
        input: true,
        ..UiDirtyFlags::default()
    };

    let detach_report = surface.detach_subtree_to_pool(child_id()).unwrap();

    assert_eq!(detach_report.recycled_count, 1);
    assert_eq!(detach_report.resident_node_count, 1);
    assert_eq!(detach_report.resident_bucket_count, 1);
    assert_eq!(
        detach_report.max_resident_node_count,
        UiSurfaceNodePool::max_resident_node_count()
    );
    assert!(surface.tree.node(child_id()).is_none());
    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
    assert_eq!(surface.focus.hovered, vec![root_id()]);
    assert_eq!(surface.input.active_pointer_capture(), None);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.input.drag_drop, None);
    assert!(surface.component_state(child_id()).is_none());

    let reuse_report = surface
        .insert_or_reuse_pooled_child(root_id(), child_node())
        .unwrap();

    assert_eq!(reuse_report.reused_count, 1);
    assert_eq!(reuse_report.resident_node_count, 0);
    assert_eq!(reuse_report.resident_bucket_count, 0);
    let child = surface
        .tree
        .node(child_id())
        .expect("pooled child should be reinserted");
    assert_eq!(child.parent, Some(root_id()));
    assert_eq!(child.node_path, UiNodePath::new("root/action"));
    assert_eq!(child.paint_order, 2);
    assert!(!child.state_flags.pressed);
    assert!(!child.state_flags.dirty);
    assert!(!child.dirty.style);
    assert!(!child.dirty.text);
    assert!(child.dirty.layout);
    assert!(child.dirty.render);
    assert!(child.dirty.hit_test);
    assert!(surface.component_state(child_id()).is_none());

    let rebuild_report = surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(rebuild_report.control_pool_recycled_count, 1);
    assert_eq!(rebuild_report.control_pool_reused_count, 1);
    assert_eq!(rebuild_report.control_pool_created_count, 0);
    assert_eq!(rebuild_report.control_pool_discarded_count, 0);
    assert!(!surface.dirty_flags().any());
}

#[test]
fn surface_node_pool_counts_created_nodes_only_when_no_matching_recycled_node_exists() {
    let mut surface = root_surface();

    let create_report = surface
        .insert_or_reuse_pooled_child(root_id(), child_node())
        .unwrap();

    assert_eq!(create_report.created_count, 1);
    assert_eq!(create_report.reused_count, 0);
    let rebuild_report = surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(rebuild_report.control_pool_created_count, 1);
    assert_eq!(rebuild_report.control_pool_reused_count, 0);
    assert_eq!(rebuild_report.control_pool_recycled_count, 0);
    assert_eq!(rebuild_report.control_pool_discarded_count, 0);

    surface
        .tree
        .node_mut(child_id())
        .expect("child should exist")
        .constraints
        .width = fixed_constraint(80.0);
    surface
        .tree
        .node_mut(child_id())
        .expect("child should exist")
        .dirty
        .layout = true;
    let resize_report = surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(resize_report.control_pool_created_count, 0);
    assert_eq!(resize_report.control_pool_reused_count, 0);
    assert_eq!(resize_report.control_pool_recycled_count, 0);
    assert_eq!(resize_report.control_pool_discarded_count, 0);
}

#[test]
fn surface_node_pool_bounds_dynamic_identities_and_reports_capacity_rejection() {
    let mut surface = pooled_surface();
    for bucket in 0..UiSurfaceNodePool::max_bucket_count() {
        for replica in 0..UiSurfaceNodePool::max_nodes_per_bucket() {
            assert!(surface.node_pool.recycle(pool_node(bucket, replica)));
        }
    }

    assert_eq!(
        surface.node_pool.resident_node_count(),
        UiSurfaceNodePool::max_resident_node_count()
    );
    assert_eq!(
        surface.node_pool.resident_bucket_count(),
        UiSurfaceNodePool::max_bucket_count()
    );

    let report = surface.detach_subtree_to_pool(child_id()).unwrap();
    assert_eq!(report.recycled_count, 0);
    assert_eq!(report.discarded_count, 1);
    assert_eq!(report.capacity_rejected_count, 1);
    assert_eq!(
        report.resident_node_count,
        UiSurfaceNodePool::max_resident_node_count()
    );
    assert_eq!(
        report.resident_bucket_count,
        UiSurfaceNodePool::max_bucket_count()
    );
}

#[test]
fn surface_node_pool_explicit_trim_releases_detached_reuse_storage() {
    let mut surface = pooled_surface();
    for bucket in 0..UiSurfaceNodePool::max_bucket_count() {
        for replica in 0..UiSurfaceNodePool::max_nodes_per_bucket() {
            assert!(surface.node_pool.recycle(pool_node(bucket, replica)));
        }
    }

    let report = surface.trim_retained_node_pool();
    assert_eq!(
        report.trimmed_node_count,
        UiSurfaceNodePool::max_resident_node_count()
    );
    assert_eq!(
        report.trimmed_bucket_count,
        UiSurfaceNodePool::max_bucket_count()
    );
    assert_eq!(report.resident_node_count, 0);
    assert_eq!(report.resident_bucket_count, 0);
}

fn pooled_surface() -> UiSurface {
    let mut surface = root_surface();
    surface.tree.insert_child(root_id(), child_node()).unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn root_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.node_pool"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root")).with_constraints(BoxConstraints {
            width: fixed_constraint(120.0),
            height: fixed_constraint(60.0),
        }),
    );
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn child_node() -> UiTreeNode {
    UiTreeNode::new(child_id(), UiNodePath::new("root/action"))
        .with_constraints(BoxConstraints {
            width: fixed_constraint(48.0),
            height: fixed_constraint(20.0),
        })
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
        })
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "Button".to_string(),
            control_id: Some("action.primary".to_string()),
            classes: vec!["primary".to_string()],
            attributes: BTreeMap::new(),
            slot_attributes: BTreeMap::new(),
            style_overrides: BTreeMap::new(),
            style_tokens: BTreeMap::new(),
            bindings: Vec::new(),
            ..Default::default()
        })
}

fn pool_node(bucket: usize, replica: usize) -> UiTreeNode {
    let mut node = child_node();
    node.node_id = UiNodeId::new(
        (10_000 + bucket * UiSurfaceNodePool::max_nodes_per_bucket() + replica) as u64,
    );
    node.node_path = UiNodePath::new(format!("pool/{bucket}"));
    node.template_metadata
        .as_mut()
        .expect("pooled fixture must retain template metadata")
        .control_id = Some(format!("pool.{bucket}"));
    node
}

fn root_id() -> UiNodeId {
    UiNodeId::new(1)
}

fn child_id() -> UiNodeId {
    UiNodeId::new(2)
}

fn root_size() -> UiSize {
    UiSize::new(120.0, 60.0)
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        preferred: size,
        max: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
