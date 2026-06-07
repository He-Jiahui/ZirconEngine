use crate::ui::surface::{hit_test_surface_frame, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, Pivot, Position, StretchMode,
        UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiMargin, UiPoint, UiSize, UiSlot,
        UiSlotKind,
    },
    surface::UiSurfaceFrame,
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn overlay_slot_z_order_mutation_rebuilds_arranged_hit_and_render_without_layout() {
    let mut surface = overlay_surface();
    let frame = surface.surface_frame();

    assert_eq!(top_hit(&frame), Some(front_id()));
    assert_eq!(arranged_z(&frame, back_id()), Some(0));
    assert_eq!(arranged_z(&frame, front_id()), Some(10));
    assert_eq!(
        slot_revision(&surface, back_id(), UiSlotKind::Overlay),
        Some(0)
    );

    assert!(!surface
        .set_overlay_slot_z_order(root_id(), front_id(), 10)
        .unwrap());
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());

    assert!(surface
        .set_overlay_slot_z_order(root_id(), back_id(), 20)
        .unwrap());
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        }
    );
    assert_eq!(
        slot_revision(&surface, back_id(), UiSlotKind::Overlay),
        Some(1)
    );
    assert_eq!(
        surface.tree.node(back_id()).unwrap().z_index,
        0,
        "slot z_order mutation must not rewrite retained node z_index"
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        report.dirty_flags,
        UiDirtyFlags {
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        }
    );
    assert_eq!(report.dirty_node_count, 1);
    assert!(!report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert!(report.hit_grid_rebuilt);
    assert!(report.render_rebuilt);

    let frame = surface.surface_frame();
    assert_eq!(arranged_z(&frame, back_id()), Some(20));
    assert_eq!(render_z(&frame, back_id()), Some(20));
    assert_eq!(hit_z(&frame, back_id()), Some(20));
    assert_eq!(top_hit(&frame), Some(back_id()));
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

#[test]
fn canvas_slot_z_order_mutation_rebuilds_arranged_hit_and_render_without_layout() {
    let mut surface = canvas_surface();
    let frame = surface.surface_frame();

    assert_eq!(top_hit(&frame), Some(front_id()));
    assert_eq!(arranged_z(&frame, back_id()), Some(0));
    assert_eq!(arranged_z(&frame, front_id()), Some(10));
    assert_eq!(
        slot_revision(&surface, back_id(), UiSlotKind::Canvas),
        Some(0)
    );

    assert!(!surface
        .set_canvas_slot_z_order(root_id(), front_id(), 10)
        .unwrap());
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());

    assert!(surface
        .set_canvas_slot_z_order(root_id(), back_id(), 20)
        .unwrap());
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        }
    );
    assert_eq!(
        slot_revision(&surface, back_id(), UiSlotKind::Canvas),
        Some(1)
    );
    assert_eq!(
        surface.tree.node(back_id()).unwrap().z_index,
        0,
        "slot z_order mutation must not rewrite retained node z_index"
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        report.dirty_flags,
        UiDirtyFlags {
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        }
    );
    assert_eq!(report.dirty_node_count, 1);
    assert!(!report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert!(report.hit_grid_rebuilt);
    assert!(report.render_rebuilt);

    let frame = surface.surface_frame();
    assert_eq!(arranged_z(&frame, back_id()), Some(20));
    assert_eq!(render_z(&frame, back_id()), Some(20));
    assert_eq!(hit_z(&frame, back_id()), Some(20));
    assert_eq!(top_hit(&frame), Some(back_id()));
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

#[test]
fn free_slot_canvas_placement_mutation_recomputes_layout_authority() {
    let mut surface = free_slot_surface();
    let initial_frame = arranged_frame(&surface.surface_frame(), canvas_child_id());

    assert_eq!(initial_frame, Some(UiFrame::new(0.0, 0.0, 40.0, 20.0)));
    assert_eq!(
        slot_revision(&surface, canvas_child_id(), UiSlotKind::Free),
        Some(0)
    );

    let next = UiCanvasSlotPlacement::new(
        Anchor::new(1.0, 0.5),
        Pivot::new(1.0, 0.5),
        Position::new(-10.0, 5.0),
    )
    .with_offset(UiMargin::new(2.0, 3.0, 80.0, 30.0));

    assert!(!surface
        .set_free_slot_canvas_placement(root_id(), canvas_child_id(), initial_canvas_placement())
        .unwrap());
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());

    assert!(surface
        .set_free_slot_canvas_placement(root_id(), canvas_child_id(), next)
        .unwrap());
    assert_eq!(
        slot_revision(&surface, canvas_child_id(), UiSlotKind::Free),
        Some(1)
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        }
    );

    let report = surface.rebuild_dirty(canvas_root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert!(report.hit_grid_rebuilt);
    assert!(report.render_rebuilt);
    assert_eq!(report.dirty_node_count, 2);
    assert_eq!(
        arranged_frame(&surface.surface_frame(), canvas_child_id()),
        Some(UiFrame::new(112.0, 43.0, 80.0, 30.0))
    );
    assert_eq!(
        render_frame(&surface.surface_frame(), canvas_child_id()),
        Some(UiFrame::new(112.0, 43.0, 80.0, 30.0))
    );
    assert_eq!(
        hit_frame(&surface.surface_frame(), canvas_child_id()),
        Some(UiFrame::new(112.0, 43.0, 80.0, 30.0))
    );
    assert_eq!(
        hit_test_surface_frame(&surface.surface_frame(), UiPoint::new(115.0, 45.0)).top_hit,
        Some(canvas_child_id())
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

#[test]
fn canvas_slot_canvas_placement_mutation_recomputes_layout_authority() {
    let mut surface = canvas_slot_surface();
    let initial_frame = arranged_frame(&surface.surface_frame(), canvas_child_id());

    assert_eq!(initial_frame, Some(UiFrame::new(0.0, 0.0, 40.0, 20.0)));
    assert_eq!(
        slot_revision(&surface, canvas_child_id(), UiSlotKind::Canvas),
        Some(0)
    );

    let next = UiCanvasSlotPlacement::new(
        Anchor::new(0.25, 0.5),
        Pivot::new(0.0, 0.5),
        Position::new(4.0, 6.0),
    )
    .with_anchor_max(Anchor::new(0.75, 0.5))
    .with_offset(UiMargin::new(8.0, 0.0, 12.0, 28.0));

    assert!(!surface
        .set_canvas_slot_canvas_placement(root_id(), canvas_child_id(), initial_canvas_placement())
        .unwrap());
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());

    assert!(surface
        .set_canvas_slot_canvas_placement(root_id(), canvas_child_id(), next)
        .unwrap());
    assert_eq!(
        slot_revision(&surface, canvas_child_id(), UiSlotKind::Canvas),
        Some(1)
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        }
    );

    let report = surface.rebuild_dirty(canvas_root_size()).unwrap();

    assert!(report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert!(report.hit_grid_rebuilt);
    assert!(report.render_rebuilt);
    assert_eq!(report.dirty_node_count, 2);
    assert_eq!(
        arranged_frame(&surface.surface_frame(), canvas_child_id()),
        Some(UiFrame::new(62.0, 42.0, 76.0, 28.0))
    );
    assert_eq!(
        render_frame(&surface.surface_frame(), canvas_child_id()),
        Some(UiFrame::new(62.0, 42.0, 76.0, 28.0))
    );
    assert_eq!(
        hit_frame(&surface.surface_frame(), canvas_child_id()),
        Some(UiFrame::new(62.0, 42.0, 76.0, 28.0))
    );
    assert_eq!(
        hit_test_surface_frame(&surface.surface_frame(), UiPoint::new(68.0, 46.0)).top_hit,
        Some(canvas_child_id())
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

fn overlay_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.surface.slot_dirty"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::Overlay)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(root_id(), pointer_node(back_id(), "root/back", 0))
        .unwrap();
    surface
        .tree
        .insert_child(root_id(), pointer_node(front_id(), "root/front", 0))
        .unwrap();
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id(), back_id(), UiSlotKind::Overlay).with_z_order(0));
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id(), front_id(), UiSlotKind::Overlay).with_z_order(10));

    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn canvas_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.surface.canvas_slot_dirty"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::Canvas)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(root_id(), pointer_node(back_id(), "root/back", 0))
        .unwrap();
    surface
        .tree
        .insert_child(root_id(), pointer_node(front_id(), "root/front", 0))
        .unwrap();
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id(), back_id(), UiSlotKind::Canvas).with_z_order(0));
    surface
        .tree
        .slots
        .push(UiSlot::new(root_id(), front_id(), UiSlotKind::Canvas).with_z_order(10));

    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn free_slot_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.surface.free_slot_dirty"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            canvas_pointer_node(canvas_child_id(), "root/canvas_child"),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(root_id(), canvas_child_id(), UiSlotKind::Free)
            .with_canvas_placement(initial_canvas_placement()),
    );

    surface.compute_layout(canvas_root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn canvas_slot_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.surface.canvas_slot_placement_dirty",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::Canvas)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            canvas_pointer_node(canvas_child_id(), "root/canvas_child"),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(root_id(), canvas_child_id(), UiSlotKind::Canvas)
            .with_canvas_placement(initial_canvas_placement()),
    );

    surface.compute_layout(canvas_root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn initial_canvas_placement() -> UiCanvasSlotPlacement {
    UiCanvasSlotPlacement::new(Anchor::default(), Pivot::default(), Position::default())
        .with_offset(UiMargin::new(0.0, 0.0, 40.0, 20.0))
}

fn canvas_pointer_node(id: UiNodeId, path: impl Into<String>) -> UiTreeNode {
    UiTreeNode::new(id, UiNodePath::new(path))
        .with_constraints(BoxConstraints::default())
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(format!("slot.dirty.{}", id.0)),
            ..Default::default()
        })
}

fn pointer_node(id: UiNodeId, path: impl Into<String>, z_index: i32) -> UiTreeNode {
    UiTreeNode::new(id, UiNodePath::new(path))
        .with_constraints(BoxConstraints {
            width: fixed_constraint(80.0),
            height: fixed_constraint(40.0),
        })
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(format!("slot.dirty.{}", id.0)),
            ..Default::default()
        })
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        ..Default::default()
    }
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

fn top_hit(frame: &UiSurfaceFrame) -> Option<UiNodeId> {
    hit_test_surface_frame(frame, UiPoint::new(8.0, 8.0)).top_hit
}

fn arranged_z(frame: &UiSurfaceFrame, node_id: UiNodeId) -> Option<i32> {
    frame
        .arranged_tree
        .get(node_id)
        .map(|arranged| arranged.z_index)
}

fn arranged_frame(frame: &UiSurfaceFrame, node_id: UiNodeId) -> Option<UiFrame> {
    frame
        .arranged_tree
        .get(node_id)
        .map(|arranged| arranged.frame)
}

fn render_frame(frame: &UiSurfaceFrame, node_id: UiNodeId) -> Option<UiFrame> {
    frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .map(|command| command.frame)
}

fn render_z(frame: &UiSurfaceFrame, node_id: UiNodeId) -> Option<i32> {
    frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .map(|command| command.z_index)
}

fn hit_frame(frame: &UiSurfaceFrame, node_id: UiNodeId) -> Option<UiFrame> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| entry.frame)
}

fn hit_z(frame: &UiSurfaceFrame, node_id: UiNodeId) -> Option<i32> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| entry.z_index)
}

fn slot_revision(surface: &UiSurface, child_id: UiNodeId, kind: UiSlotKind) -> Option<u64> {
    surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.parent_id == root_id() && slot.child_id == child_id && slot.kind == kind)
        .map(|slot| slot.dirty_revision)
}

fn root_id() -> UiNodeId {
    UiNodeId::new(1)
}

fn back_id() -> UiNodeId {
    UiNodeId::new(2)
}

fn front_id() -> UiNodeId {
    UiNodeId::new(3)
}

fn canvas_child_id() -> UiNodeId {
    UiNodeId::new(4)
}

fn root_size() -> UiSize {
    UiSize::new(120.0, 80.0)
}

fn canvas_root_size() -> UiSize {
    UiSize::new(200.0, 100.0)
}
