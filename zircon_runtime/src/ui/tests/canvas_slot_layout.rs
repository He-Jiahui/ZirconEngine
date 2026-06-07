use crate::ui::surface::{hit_test_surface_frame, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, Pivot, Position, StretchMode,
        UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiMargin, UiPoint, UiSize, UiSlot,
        UiSlotKind,
    },
    surface::UiCanvasLayerGroup,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
};

#[test]
fn free_canvas_slot_stretches_between_min_and_max_anchors() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.canvas.slot.stretch"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::Free)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            pointer_node(child_id(), "root/stretch_child", BoxConstraints::default()),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(root_id(), child_id(), UiSlotKind::Free).with_canvas_placement(
            UiCanvasSlotPlacement::new(
                Anchor::new(0.25, 0.0),
                Pivot::new(1.0, 1.0),
                Position::new(5.0, 2.0),
            )
            .with_anchor_max(Anchor::new(0.75, 1.0))
            .with_offset(UiMargin::new(10.0, 4.0, 15.0, 6.0)),
        ),
    );

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();
    let frame = surface.surface_frame();
    let stretched = frame
        .arranged_tree
        .get(child_id())
        .expect("stretched canvas child should be arranged");

    assert_eq!(stretched.frame, UiFrame::new(65.0, 6.0, 70.0, 88.0));
    assert_eq!(
        surface.tree.node(child_id()).unwrap().layout_cache.frame,
        stretched.frame
    );
    assert_eq!(render_frame_for(&frame), Some(stretched.frame));
    assert_eq!(hit_frame_for(&frame), Some(stretched.frame));
    assert_eq!(
        hit_test_surface_frame(&frame, UiPoint::new(70.0, 12.0)).top_hit,
        Some(child_id())
    );
}

#[test]
fn free_canvas_slot_can_stretch_one_axis_and_keep_pivot_on_the_other() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.canvas.slot.partial_stretch"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root")).with_container(UiContainerKind::Free),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            pointer_node(
                child_id(),
                "root/partial_stretch_child",
                BoxConstraints {
                    width: AxisConstraint::default(),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(root_id(), child_id(), UiSlotKind::Free).with_canvas_placement(
            UiCanvasSlotPlacement::new(
                Anchor::new(0.0, 0.5),
                Pivot::new(0.0, 0.5),
                Position::new(2.0, 4.0),
            )
            .with_anchor_max(Anchor::new(1.0, 0.5))
            .with_offset(UiMargin::new(8.0, 0.0, 12.0, 60.0)),
        ),
    );

    surface.compute_layout(UiSize::new(160.0, 80.0)).unwrap();

    assert_eq!(
        surface.tree.node(child_id()).unwrap().layout_cache.frame,
        UiFrame::new(10.0, 34.0, 138.0, 20.0)
    );
}

#[test]
fn canvas_container_canvas_slot_drives_layout_render_hit_and_z_order() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.canvas.slot.container"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::Canvas)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            root_id(),
            pointer_node(
                child_id(),
                "root/canvas_child",
                BoxConstraints {
                    width: fixed_constraint(32.0),
                    height: fixed_constraint(18.0),
                },
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(root_id(), child_id(), UiSlotKind::Canvas)
            .with_canvas_placement(
                UiCanvasSlotPlacement::new(
                    Anchor::new(0.5, 0.25),
                    Pivot::new(0.5, 0.5),
                    Position::new(4.0, 6.0),
                )
                .with_offset(UiMargin::new(10.0, 2.0, 80.0, 40.0)),
            )
            .with_z_order(6),
    );

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();
    let frame = surface.surface_frame();
    let arranged = frame
        .arranged_tree
        .get(child_id())
        .expect("canvas child should be arranged by its Canvas slot");

    assert_eq!(arranged.frame, UiFrame::new(74.0, 13.0, 80.0, 40.0));
    assert_eq!(arranged.z_index, 6);
    assert_eq!(render_frame_for(&frame), Some(arranged.frame));
    assert_eq!(hit_frame_for(&frame), Some(arranged.frame));
    assert_eq!(
        hit_test_surface_frame(&frame, UiPoint::new(80.0, 20.0)).top_hit,
        Some(child_id())
    );
}

#[test]
fn canvas_container_groups_same_z_order_children_into_one_layer() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.canvas.slot.layers"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_container(UiContainerKind::Canvas)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    for id in [UiNodeId::new(2), UiNodeId::new(3), UiNodeId::new(4)] {
        surface
            .tree
            .insert_child(
                root_id(),
                pointer_node(
                    id,
                    format!("root/canvas_child_{}", id.0),
                    BoxConstraints {
                        width: fixed_constraint(20.0),
                        height: fixed_constraint(12.0),
                    },
                ),
            )
            .unwrap();
    }
    surface
        .tree
        .insert_child(
            root_id(),
            pointer_node(
                UiNodeId::new(5),
                "root/canvas_child_hidden",
                BoxConstraints {
                    width: fixed_constraint(20.0),
                    height: fixed_constraint(12.0),
                },
            )
            .with_visibility(UiVisibility::Hidden),
        )
        .unwrap();
    surface.tree.slots.push(canvas_slot(UiNodeId::new(2), 4));
    surface.tree.slots.push(canvas_slot(UiNodeId::new(3), 4));
    surface.tree.slots.push(canvas_slot(UiNodeId::new(4), 9));
    surface.tree.slots.push(canvas_slot(UiNodeId::new(5), 4));

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();
    let frame = surface.surface_frame();

    assert_eq!(
        frame.arranged_tree.canvas_layers,
        vec![
            UiCanvasLayerGroup {
                parent_id: root_id(),
                layer_index: 0,
                z_order: 4,
                child_ids: vec![UiNodeId::new(2), UiNodeId::new(3)],
            },
            UiCanvasLayerGroup {
                parent_id: root_id(),
                layer_index: 1,
                z_order: 9,
                child_ids: vec![UiNodeId::new(4)],
            },
        ]
    );
    assert_eq!(
        frame.arranged_tree.draw_order,
        vec![
            root_id(),
            UiNodeId::new(2),
            UiNodeId::new(3),
            UiNodeId::new(5),
            UiNodeId::new(4)
        ]
    );
    let rendered_node_ids: Vec<_> = frame
        .render_extract
        .list
        .commands
        .iter()
        .map(|command| command.node_id)
        .collect();
    assert!(rendered_node_ids.contains(&root_id()));
    assert!(rendered_node_ids.contains(&UiNodeId::new(2)));
    assert!(rendered_node_ids.contains(&UiNodeId::new(3)));
    assert!(rendered_node_ids.contains(&UiNodeId::new(4)));
    assert!(!rendered_node_ids.contains(&UiNodeId::new(5)));
}

fn pointer_node(id: UiNodeId, path: impl Into<String>, constraints: BoxConstraints) -> UiTreeNode {
    UiTreeNode::new(id, UiNodePath::new(path))
        .with_constraints(constraints)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            ..Default::default()
        })
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(format!("canvas.slot.{}", id.0)),
            ..Default::default()
        })
}

fn canvas_slot(child_id: UiNodeId, z_order: i32) -> UiSlot {
    UiSlot::new(root_id(), child_id, UiSlotKind::Canvas)
        .with_canvas_placement(UiCanvasSlotPlacement::new(
            Anchor::new(0.0, 0.0),
            Pivot::new(0.0, 0.0),
            Position::new(0.0, 0.0),
        ))
        .with_z_order(z_order)
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

fn render_frame_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
) -> Option<UiFrame> {
    frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == child_id())
        .map(|command| command.frame)
}

fn hit_frame_for(frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame) -> Option<UiFrame> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == child_id())
        .map(|entry| entry.frame)
}

fn root_id() -> UiNodeId {
    UiNodeId::new(1)
}

fn child_id() -> UiNodeId {
    UiNodeId::new(2)
}
