use crate::ui::surface::{hit_test_surface_frame, UiSurface};
use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, DesiredSize, StretchMode, UiAlignment, UiAlignment2D,
        UiAxis, UiContainerKind, UiFrame, UiGridBoxConfig, UiGridSlotPlacement, UiLinearBoxConfig,
        UiMargin, UiPoint, UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize,
        UiSlot, UiSlotKind, UiVirtualListConfig, UiVirtualListWindow,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn linear_layout_consumes_slot_padding_order_and_alignment() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.linear"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 4.0 }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/later")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(20.0),
                    height: fixed_constraint(10.0),
                },
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/first")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: AxisConstraint::default(),
                },
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Linear)
            .with_padding(UiMargin::new(5.0, 4.0, 7.0, 6.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End))
            .with_order(2),
    );
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Linear)
            .with_padding(UiMargin::new(1.0, 2.0, 3.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Fill))
            .with_order(1),
    );

    surface.compute_layout(UiSize::new(200.0, 60.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .desired_size,
        DesiredSize::new(70.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(1.0, 2.0, 30.0, 54.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(43.0, 44.0, 20.0, 10.0)
    );
}

#[test]
fn free_layout_consumes_explicit_slot_padding_alignment_and_preserves_default_anchor_fallback() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.free"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/anchored"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(20.0),
                })
                .with_anchor(zircon_runtime_interface::ui::layout::Anchor::new(0.5, 0.5))
                .with_pivot(zircon_runtime_interface::ui::layout::Pivot::new(0.5, 0.5))
                .with_position(zircon_runtime_interface::ui::layout::Position::new(
                    10.0, -5.0,
                )),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/slotted"))
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(10.0),
                })
                .with_position(zircon_runtime_interface::ui::layout::Position::new(
                    2.0, 3.0,
                )),
        )
        .unwrap();
    surface.tree.slots.push(UiSlot::new(
        UiNodeId::new(1),
        UiNodeId::new(2),
        UiSlotKind::Free,
    ));
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Free)
            .with_padding(UiMargin::new(10.0, 5.0, 20.0, 15.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::End, UiAlignment::Center)),
    );

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(95.0, 35.0, 30.0, 20.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(142.0, 43.0, 40.0, 10.0)
    );
}

#[test]
fn overlay_layout_consumes_slot_padding_alignment() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.overlay"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::Overlay),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/badge")).with_constraints(
                BoxConstraints {
                    width: AxisConstraint::default(),
                    height: fixed_constraint(12.0),
                },
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Overlay)
            .with_padding(UiMargin::new(8.0, 6.0, 12.0, 10.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Fill, UiAlignment::End)),
    );

    surface.compute_layout(UiSize::new(160.0, 80.0)).unwrap();

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(8.0, 58.0, 140.0, 12.0)
    );
}

#[test]
fn overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.overlay.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::Overlay)
            .with_clip_to_bounds(true)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/back",
                "back.overlay",
                BoxConstraints {
                    width: fixed_constraint(90.0),
                    height: fixed_constraint(50.0),
                },
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                3,
                "root/front",
                "front.overlay",
                BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                },
                10,
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Overlay)
            .with_padding(UiMargin::new(4.0, 4.0, 4.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Start)),
    );
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Overlay)
            .with_padding(UiMargin::new(10.0, 8.0, 10.0, 8.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::End, UiAlignment::End)),
    );

    surface.compute_layout(UiSize::new(120.0, 80.0)).unwrap();
    let frame = surface.surface_frame();
    let front = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("front overlay child should be arranged");

    assert_eq!(front.frame, UiFrame::new(70.0, 52.0, 40.0, 20.0));
    assert_eq!(front.clip_frame, UiFrame::new(70.0, 52.0, 40.0, 20.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(3)),
        Some(front.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(3)), Some(front.frame));
    assert_eq!(hit_z_for(&frame, UiNodeId::new(3)), Some(front.z_index));

    let hit = hit_test_surface_frame(&frame, UiPoint::new(75.0, 53.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(hit.stacked, vec![UiNodeId::new(3), UiNodeId::new(2)]);
    assert_eq!(
        hit.path.bubble_route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
}

#[test]
fn scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.scroll.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                gap: 0.0,
                scrollbar_visibility: UiScrollbarVisibility::Auto,
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 40.0,
                    overscan: 0,
                }),
            }))
            .with_scroll_state(UiScrollState {
                offset: 80.0,
                viewport_extent: 0.0,
                content_extent: 0.0,
            })
            .with_clip_to_bounds(true)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                pointer_node(
                    10 + item,
                    format!("root/item_{item}"),
                    format!("scroll.item.{item}"),
                    BoxConstraints {
                        width: fixed_constraint(200.0),
                        height: fixed_constraint(40.0),
                    },
                    item as i32,
                ),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();
    let frame = surface.surface_frame();
    let visible = frame
        .arranged_tree
        .get(UiNodeId::new(12))
        .expect("scrolled item should be arranged in the virtual window");

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .layout_cache
            .virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 2,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(visible.frame, UiFrame::new(0.0, 0.0, 200.0, 40.0));
    assert_eq!(visible.clip_frame, UiFrame::new(0.0, 0.0, 200.0, 40.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(12)),
        Some(visible.frame)
    );
    assert_eq!(
        hit_frame_for(&frame, UiNodeId::new(12)),
        Some(visible.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(10)), None);

    let hit = hit_test_surface_frame(&frame, UiPoint::new(20.0, 20.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(12)));
}

#[test]
fn wrap_flow_slot_padding_alignment_feeds_shared_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.flow.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::WrapBox(zircon_runtime_interface::ui::layout::UiWrapBoxConfig {
                horizontal_gap: 5.0,
                vertical_gap: 3.0,
                item_min_width: 20.0,
            }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/flow_first",
                "flow.first",
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(10.0),
                },
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                3,
                "root/flow_second",
                "flow.second",
                BoxConstraints {
                    width: fixed_constraint(25.0),
                    height: fixed_constraint(12.0),
                },
                5,
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Flow)
            .with_padding(UiMargin::new(2.0, 1.0, 4.0, 3.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End))
            .with_order(2),
    );
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Flow)
            .with_padding(UiMargin::new(1.0, 2.0, 1.0, 2.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Fill))
            .with_order(1),
    );

    surface.compute_layout(UiSize::new(70.0, 40.0)).unwrap();
    let frame = surface.surface_frame();
    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("flow child should be arranged");
    let second = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("flow child should be arranged");

    assert_eq!(second.frame, UiFrame::new(1.0, 2.0, 25.0, 12.0));
    assert_eq!(first.frame, UiFrame::new(34.0, 3.0, 30.0, 10.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(2)),
        Some(first.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(2)), Some(first.frame));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(3)),
        Some(second.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(3)), Some(second.frame));

    let hit = hit_test_surface_frame(&frame, UiPoint::new(4.0, 5.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
}

#[test]
fn grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.slot.grid.frame"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::GridBox(UiGridBoxConfig {
                columns: 2,
                rows: 2,
                column_gap: 4.0,
                row_gap: 6.0,
            }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                2,
                "root/grid_a",
                "grid.a",
                BoxConstraints {
                    width: fixed_constraint(30.0),
                    height: fixed_constraint(20.0),
                },
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(
                3,
                "root/grid_b",
                "grid.b",
                BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                },
                5,
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(0, 0))
            .with_padding(UiMargin::new(2.0, 2.0, 2.0, 2.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::Center)),
    );
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(3), UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1))
            .with_padding(UiMargin::new(4.0, 4.0, 4.0, 4.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Start, UiAlignment::Start)),
    );

    surface.compute_layout(UiSize::new(124.0, 82.0)).unwrap();
    let frame = surface.surface_frame();
    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("grid child should be arranged");
    let second = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("grid child should be arranged");

    assert_eq!(first.frame, UiFrame::new(15.0, 9.0, 30.0, 20.0));
    assert_eq!(second.frame, UiFrame::new(68.0, 48.0, 40.0, 20.0));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(2)),
        Some(first.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(2)), Some(first.frame));
    assert_eq!(
        render_frame_for(&frame, UiNodeId::new(3)),
        Some(second.frame)
    );
    assert_eq!(hit_frame_for(&frame, UiNodeId::new(3)), Some(second.frame));

    let hit = hit_test_surface_frame(&frame, UiPoint::new(70.0, 50.0));
    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
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

fn pointer_node(
    id: u64,
    path: impl Into<String>,
    control_id: impl Into<String>,
    constraints: BoxConstraints,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(path))
        .with_constraints(constraints)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.into()),
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

fn render_frame_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<UiFrame> {
    frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .map(|command| command.frame)
}

fn hit_frame_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<UiFrame> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| entry.frame)
}

fn hit_z_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<i32> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| entry.z_index)
}
