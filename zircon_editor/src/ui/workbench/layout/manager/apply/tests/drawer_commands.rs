use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, LayoutManager, WorkbenchLayout,
};
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn activating_a_drawer_collapses_the_other_drawer_in_the_same_region() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let hierarchy = ViewInstanceId::new("editor.hierarchy#region");
    let plugins = ViewInstanceId::new("editor.module_plugins#region");
    let window = layout
        .active_activity_window_mut()
        .expect("active workbench window");

    let left_top = window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("left-top drawer");
    left_top.tab_stack.tabs = vec![hierarchy.clone()];
    left_top.tab_stack.active_tab = Some(hierarchy.clone());
    left_top.active_view = Some(hierarchy.clone());
    left_top.mode = ActivityDrawerMode::Pinned;

    let left_bottom = window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftBottom)
        .expect("left-bottom drawer");
    left_bottom.tab_stack.tabs = vec![plugins.clone()];
    left_bottom.mode = ActivityDrawerMode::Collapsed;

    assert!(
        manager
            .apply(
                &mut layout,
                LayoutCommand::ActivateDrawerTab {
                    slot: ActivityDrawerSlot::LeftBottom,
                    instance_id: plugins.clone(),
                },
            )
            .expect("activate left-bottom drawer")
            .changed
    );

    let drawers = layout.active_activity_window_drawers();
    let left_top = &drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(left_top.mode, ActivityDrawerMode::Collapsed);
    assert_eq!(left_top.tab_stack.active_tab, None);
    assert_eq!(left_top.active_view, None);

    let left_bottom = &drawers[&ActivityDrawerSlot::LeftBottom];
    assert_eq!(left_bottom.mode, ActivityDrawerMode::Pinned);
    assert_eq!(left_bottom.tab_stack.active_tab, Some(plugins.clone()));
    assert_eq!(left_bottom.active_view, Some(plugins));

    assert!(
        manager
            .apply(
                &mut layout,
                LayoutCommand::FocusView {
                    instance_id: hierarchy.clone(),
                },
            )
            .expect("focus left-top drawer")
            .changed
    );
    let drawers = layout.active_activity_window_drawers();
    assert_eq!(
        drawers[&ActivityDrawerSlot::LeftTop].active_view,
        Some(hierarchy)
    );
    assert_eq!(
        drawers[&ActivityDrawerSlot::LeftBottom].mode,
        ActivityDrawerMode::Collapsed
    );
}

#[test]
fn drawer_extent_is_a_local_geometry_patch() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let unrelated_active = ViewInstanceId::new("editor.unrelated#stale");
    let window = layout
        .active_activity_window_mut()
        .expect("active workbench window");
    let unrelated = window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::RightTop)
        .expect("right-top drawer");
    unrelated.mode = ActivityDrawerMode::Pinned;
    unrelated.tab_stack.tabs.clear();
    unrelated.tab_stack.active_tab = Some(unrelated_active.clone());
    unrelated.active_view = Some(unrelated_active.clone());

    assert!(
        manager
            .apply(
                &mut layout,
                LayoutCommand::SetDrawerExtent {
                    slot: ActivityDrawerSlot::LeftTop,
                    extent: 344.0,
                },
            )
            .expect("drawer extent")
            .changed
    );

    {
        let window = layout
            .active_activity_window_mut()
            .expect("active workbench window");
        assert_eq!(
            window.activity_drawers[&ActivityDrawerSlot::LeftTop].extent,
            344.0
        );
        let unrelated = &window.activity_drawers[&ActivityDrawerSlot::RightTop];
        assert_eq!(
            unrelated.tab_stack.active_tab,
            Some(unrelated_active.clone())
        );
        assert_eq!(unrelated.active_view, Some(unrelated_active));
    }
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftTop].extent,
        344.0
    );
}

#[test]
fn drawer_region_extent_updates_both_region_slots_in_one_transaction() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();

    assert!(
        manager
            .apply(
                &mut layout,
                LayoutCommand::SetDrawerRegionExtent {
                    slot: ActivityDrawerSlot::RightBottom,
                    extent: 376.0,
                },
            )
            .expect("drawer region extent")
            .changed
    );

    {
        let window = layout
            .active_activity_window_mut()
            .expect("active workbench window");
        assert_eq!(
            window.activity_drawers[&ActivityDrawerSlot::RightTop].extent,
            376.0
        );
        assert_eq!(
            window.activity_drawers[&ActivityDrawerSlot::RightBottom].extent,
            376.0
        );
    }
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::RightTop].extent,
        376.0
    );
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::RightBottom].extent,
        376.0
    );
}

#[test]
fn drawer_region_extent_validates_all_slots_before_mutating_any_slot() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let previous = {
        let window = layout
            .active_activity_window_mut()
            .expect("active workbench window");
        window
            .activity_drawers
            .remove(&ActivityDrawerSlot::LeftBottom);
        window.activity_drawers[&ActivityDrawerSlot::LeftTop].extent
    };

    assert!(manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerRegionExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: previous + 100.0,
            },
        )
        .is_err());

    let window = layout
        .active_activity_window_mut()
        .expect("active workbench window");
    assert_eq!(
        window.activity_drawers[&ActivityDrawerSlot::LeftTop].extent,
        previous
    );
}
