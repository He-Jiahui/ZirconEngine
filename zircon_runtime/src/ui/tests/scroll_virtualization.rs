use crate::ui::{
    layout::plan_scrollable_virtual_window, surface::UiSurface, tree::UiRuntimeTreeScrollExt,
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
        UiVirtualListWindow,
    },
    tree::{UiInputPolicy, UiStateFlags, UiTreeNode},
};

#[test]
fn retained_virtual_list_only_arranges_visible_window() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.virtual.window"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(200.0),
                height: fixed_constraint(80.0),
            })
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
            }),
    );

    for item in 0..6 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(200.0),
                    height: fixed_constraint(40.0),
                }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();

    let scroll = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert_eq!(
        scroll.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 2,
            last_visible_exclusive: 4,
        })
    );
    assert_eq!(
        frame_for(&surface, 10),
        UiFrame::default(),
        "item before the virtual window should not be arranged"
    );
    assert_eq!(frame_for(&surface, 12), UiFrame::new(0.0, 0.0, 200.0, 40.0));
    assert_eq!(
        frame_for(&surface, 13),
        UiFrame::new(0.0, 40.0, 200.0, 40.0)
    );
    assert_eq!(
        frame_for(&surface, 14),
        UiFrame::default(),
        "item after the virtual window should not be arranged"
    );
}

#[test]
fn scroll_offset_invalidates_virtualization_window() {
    let config = UiScrollableBoxConfig {
        axis: UiAxis::Vertical,
        gap: 0.0,
        scrollbar_visibility: UiScrollbarVisibility::Auto,
        virtualization: Some(UiVirtualListConfig {
            item_extent: 40.0,
            overscan: 1,
        }),
    };
    let previous_state = UiScrollState {
        offset: 45.0,
        viewport_extent: 80.0,
        content_extent: 240.0,
    };
    let previous_window = Some(UiVirtualListWindow {
        first_visible: 0,
        last_visible_exclusive: 5,
    });

    let same_window = plan_scrollable_virtual_window(
        config,
        previous_state,
        previous_window,
        50.0,
        6,
        80.0,
        240.0,
    );
    assert_eq!(same_window.virtual_window, previous_window);
    assert!(!same_window.visible_range_changed);

    let scrolled = plan_scrollable_virtual_window(
        config,
        previous_state,
        previous_window,
        120.0,
        6,
        80.0,
        240.0,
    );
    assert_eq!(
        scrolled.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 2,
            last_visible_exclusive: 6,
        })
    );
    assert!(scrolled.visible_range_changed);

    let viewport_changed = plan_scrollable_virtual_window(
        config,
        previous_state,
        previous_window,
        0.0,
        6,
        120.0,
        240.0,
    );
    assert!(viewport_changed.visible_range_changed);

    let content_changed = plan_scrollable_virtual_window(
        config,
        previous_state,
        previous_window,
        0.0,
        6,
        80.0,
        280.0,
    );
    assert!(content_changed.visible_range_changed);
}

#[test]
fn non_virtualized_scroll_offset_keeps_full_window_dirty_domain() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.scroll.full_window"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(200.0),
                height: fixed_constraint(80.0),
            })
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                gap: 0.0,
                scrollbar_visibility: UiScrollbarVisibility::Auto,
                virtualization: None,
            }))
            .with_scroll_state(UiScrollState::default()),
    );

    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(200.0),
                    height: fixed_constraint(40.0),
                }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();
    surface
        .tree
        .set_scroll_offset(UiNodeId::new(1), 40.0)
        .unwrap();

    let scroll = surface.tree.node(UiNodeId::new(1)).unwrap();
    assert!(scroll.dirty.layout);
    assert!(scroll.dirty.hit_test);
    assert!(scroll.dirty.render);
    assert!(!scroll.dirty.input);
    assert!(!scroll.dirty.visible_range);
}

#[test]
fn virtual_scroll_patches_arranged_and_hit_without_index_fallback() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.virtual.incremental-scroll"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(200.0),
                height: fixed_constraint(80.0),
            })
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                gap: 0.0,
                scrollbar_visibility: UiScrollbarVisibility::Auto,
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 40.0,
                    overscan: 0,
                }),
            }))
            .with_scroll_state(UiScrollState::default()),
    );

    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(10 + item),
                    UiNodePath::new(format!("root/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(200.0),
                    height: fixed_constraint(40.0),
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    ..UiStateFlags::default()
                }),
            )
            .unwrap();
    }

    surface.compute_layout(UiSize::new(200.0, 80.0)).unwrap();
    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 20.0)).top_hit,
        Some(UiNodeId::new(10))
    );
    surface
        .tree
        .set_scroll_offset(UiNodeId::new(1), 40.0)
        .unwrap();

    let report = surface.rebuild_dirty(UiSize::new(200.0, 80.0)).unwrap();

    assert!(report.arranged_outer_node_visit_count < surface.tree.nodes.len());
    assert!(report.hit_grid_outer_node_visit_count < surface.tree.nodes.len());
    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 20.0)).top_hit,
        Some(UiNodeId::new(11))
    );
    assert_eq!(
        surface.hit_test(UiPoint::new(20.0, 60.0)).top_hit,
        Some(UiNodeId::new(12))
    );
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 0,
        weight: 0.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn frame_for(surface: &UiSurface, node_id: u64) -> UiFrame {
    surface
        .tree
        .node(UiNodeId::new(node_id))
        .unwrap()
        .layout_cache
        .frame
}
