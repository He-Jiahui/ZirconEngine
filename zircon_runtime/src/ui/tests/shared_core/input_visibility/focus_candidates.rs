use super::*;

#[test]
fn focus_navigation_and_scroll_candidates_use_effective_visibility() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/visible_focus"))
                .with_frame(UiFrame::new(0.0, 0.0, 40.0, 20.0))
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(3),
                UiNodePath::new("root/hit_test_invisible_focus"),
            )
            .with_frame(UiFrame::new(50.0, 0.0, 40.0, 20.0))
            .with_visibility(UiVisibility::HitTestInvisible)
            .with_state_flags(pointer_state()),
        )
        .unwrap();
    let mut legacy_hidden_state = pointer_state();
    legacy_hidden_state.visible = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(4),
                UiNodePath::new("root/legacy_hidden_focus"),
            )
            .with_frame(UiFrame::new(100.0, 0.0, 40.0, 20.0))
            .with_state_flags(legacy_hidden_state),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(5), UiNodePath::new("root/scroll"))
                .with_frame(UiFrame::new(0.0, 40.0, 100.0, 50.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState::default())
                .with_visibility(UiVisibility::SelfHitTestInvisible)
                .with_state_flags({
                    let mut state = pointer_state();
                    state.focusable = false;
                    state
                }),
        )
        .unwrap();

    assert_eq!(surface.focus_node(UiNodeId::new(3)), Ok(()));
    assert!(surface.focus_node(UiNodeId::new(4)).is_err());
    let focus_order = surface.tree.focusable_nodes_in_navigation_order().unwrap();
    assert!(focus_order.contains(&UiNodeId::new(2)));
    assert!(focus_order.contains(&UiNodeId::new(3)));
    assert!(!focus_order.contains(&UiNodeId::new(4)));
    assert_eq!(
        surface
            .tree
            .first_scrollable_in_candidates(&[UiNodeId::new(5)])
            .unwrap(),
        Some(UiNodeId::new(5))
    );
}
