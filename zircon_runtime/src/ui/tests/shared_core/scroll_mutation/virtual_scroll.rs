use super::*;

#[test]
fn virtual_list_window_tracks_visible_range_with_overscan() {
    let window = compute_virtual_list_window(120.0, 150.0, 50.0, 20, 1);
    assert_eq!(
        window,
        UiVirtualListWindow {
            first_visible: 1,
            last_visible_exclusive: 7,
        }
    );

    let clamped = compute_virtual_list_window(0.0, 40.0, 50.0, 2, 3);
    assert_eq!(
        clamped,
        UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 2,
        }
    );
}

#[test]
fn scrollable_box_tracks_content_metrics_virtual_window_and_local_scroll_invalidation() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven)
            .with_constraints(BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 1,
                    }),
                }))
                .with_scroll_state(UiScrollState::default()),
        )
        .unwrap();

    for item in 0..5 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: false,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(scroll.layout_cache.content_size, UiSize::new(200.0, 200.0));
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(
        scroll.scroll_state,
        Some(UiScrollState {
            offset: 0.0,
            viewport_extent: 90.0,
            content_extent: 200.0,
        })
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 200.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(14))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );

    surface
        .tree
        .set_scroll_offset(UiNodeId::new(2), 80.0)
        .unwrap();

    let root = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert!(!root.dirty.layout);

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert!(scroll.dirty.layout);
    assert!(scroll.dirty.hit_test);
    assert!(scroll.dirty.render);
    assert!(scroll.dirty.visible_range);
    assert_eq!(scroll.scroll_state.unwrap().offset, 80.0);

    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 1,
            last_visible_exclusive: 5,
        })
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::default()
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(11))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, -40.0, 200.0, 40.0)
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(12))
            .unwrap()
            .layout_cache
            .frame,
        UiFrame::new(0.0, 0.0, 200.0, 40.0)
    );
}
