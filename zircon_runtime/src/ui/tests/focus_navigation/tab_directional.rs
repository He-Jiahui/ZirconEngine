use super::*;

#[test]
fn tab_navigation_uses_index_order_and_modal_group_trap() {
    let mut surface = navigation_surface();
    surface.focus_node(id(2)).unwrap();
    let navigation_index_generation = surface.navigation_index_build_generation();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(3)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(2)));

    surface.focus_node(id(5)).unwrap();
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(6)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Previous,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
    assert_eq!(
        surface.navigation_index_build_generation(),
        navigation_index_generation,
        "navigation events must query the rebuild-owned index without rebuilding it",
    );
}

#[test]
fn tab_navigation_crosses_non_modal_groups_by_group_order() {
    let mut surface = non_modal_group_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn render_only_rebuild_reuses_navigation_index_generation() {
    let mut surface = navigation_surface();
    let root_size = UiSize::new(180.0, 120.0);
    surface.rebuild_authored_frames(root_size);
    let navigation_index_generation = surface.navigation_index_build_generation();

    surface.focus_node(id(2)).unwrap();
    let report = surface.rebuild_dirty(root_size).unwrap();

    assert!(report.render_rebuilt);
    assert!(!report.layout_recomputed);
    assert_eq!(
        surface.navigation_index_build_generation(),
        navigation_index_generation,
        "focus-only render dirtiness must not rebuild navigation candidates",
    );
}

#[test]
fn text_rebuild_reuses_navigation_index_when_geometry_and_semantics_are_stable() {
    let mut surface = navigation_surface();
    let root_size = UiSize::new(180.0, 120.0);
    surface.rebuild_authored_frames(root_size);
    let navigation_index_generation = surface.navigation_index_build_generation();

    surface
        .mark_node_dirty(
            id(2),
            UiDirtyFlags {
                text: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let report = surface.rebuild_dirty(root_size).unwrap();

    assert!(report.layout_recomputed);
    assert_eq!(
        surface.navigation_index_build_generation(),
        navigation_index_generation,
        "stable text/layout work must not rebuild navigation candidates",
    );
}

#[test]
fn surface_geometry_patch_preserves_navigation_generation_and_ordering_rebuilds() {
    let mut surface = navigation_surface();
    let root_size = UiSize::new(180.0, 120.0);
    surface.rebuild_authored_frames(root_size);
    let initial_generation = surface.navigation_index_build_generation();

    surface.tree.node_mut(id(2)).unwrap().constraints.width =
        zircon_runtime_interface::ui::layout::AxisConstraint {
            min: 0.0,
            max: 48.0,
            preferred: 48.0,
            priority: 0,
            weight: 1.0,
            stretch_mode: zircon_runtime_interface::ui::layout::StretchMode::Fixed,
        };
    surface
        .mark_node_dirty(
            id(2),
            UiDirtyFlags {
                layout: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let geometry_report = surface.rebuild_dirty(root_size).unwrap();

    assert!(geometry_report.layout_recomputed);
    assert_eq!(
        surface.navigation_index_build_generation(),
        initial_generation,
        "frame-only candidate movement must patch the retained navigation index",
    );

    surface.tree.node_mut(id(2)).unwrap().z_index = 9;
    surface
        .mark_node_dirty(
            id(2),
            UiDirtyFlags {
                layout: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();
    let ordering_report = surface.rebuild_dirty(root_size).unwrap();

    assert!(ordering_report.arranged_rebuilt);
    assert!(
        surface.navigation_index_build_generation() > initial_generation,
        "ordering changes must fail closed to a complete navigation-index rebuild",
    );
}

#[test]
fn directional_navigation_honors_manual_overrides_and_blocked_edges() {
    let mut surface = navigation_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Left,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn modal_directional_navigation_rejects_manual_targets_outside_modal_group() {
    let mut surface = navigation_surface();
    let modal_b = surface.tree.nodes.get_mut(&id(6)).unwrap();
    modal_b.navigation.directional = Some(UiDirectionalNavigation {
        right: UiDirectionalNavigationTarget::Node(id(2)),
        ..Default::default()
    });
    surface.focus_node(id(6)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(6)));
}
