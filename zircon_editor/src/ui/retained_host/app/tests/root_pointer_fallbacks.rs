use super::support::*;
use crate::ui::retained_host::welcome_recent_geometry::{
    welcome_recent_row_geometry, welcome_recent_viewport,
};
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn root_welcome_recent_pointer_click_uses_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_welcome_recent_projection");
    harness.stage_missing_recent_project("E:/Missing/RecentProject", "RecentProject");

    let remove = {
        let host = harness.host.borrow();
        let pane = host
            .template_bridge
            .control_frame(callback_dispatch::PANE_SURFACE_CONTROL_ID)
            .expect("root projection should expose the Welcome pane surface frame");
        welcome_recent_row_geometry(
            welcome_recent_viewport(UiSize::new(pane.width, pane.height)),
            0,
            0.0,
        )
        .remove
    };

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_welcome_recent_pointer_clicked(
            remove.x + remove.width * 0.5,
            remove.y + remove.height * 0.5,
            0.0,
            0.0,
        );

    let host = harness.host.borrow();
    assert!(host.welcome_recent_pointer_size.width > 0.0);
    assert!(host.welcome_recent_pointer_size.height > 0.0);
    assert!(
        host.runtime
            .chrome_snapshot()
            .welcome
            .recent_projects
            .is_empty()
    );
    assert_eq!(
        host.runtime.editor_snapshot().status_line,
        "Removed recent project E:/Missing/RecentProject"
    );
}

#[test]
fn root_welcome_recent_pointer_move_prefers_cached_size_over_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_welcome_recent_cached_move");
    harness.stage_missing_recent_project("E:/Missing/RecentProject", "RecentProject");
    {
        let mut host = harness.host.borrow_mut();
        host.welcome_recent_pointer_size = UiSize::new(321.0, 222.0);
    }

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_welcome_recent_pointer_moved(160.0, 204.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(host.welcome_recent_pointer_size, UiSize::new(321.0, 222.0));
}

#[test]
fn root_welcome_recent_pointer_scroll_prefers_cached_size_over_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_welcome_recent_cached_scroll");
    harness.stage_missing_recent_project("E:/Missing/RecentProject", "RecentProject");
    {
        let mut host = harness.host.borrow_mut();
        host.welcome_recent_pointer_size = UiSize::new(321.0, 222.0);
    }

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_welcome_recent_pointer_scrolled(160.0, 204.0, 24.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(host.welcome_recent_pointer_size, UiSize::new(321.0, 222.0));
}

#[test]
fn root_hierarchy_pointer_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_hierarchy_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_hierarchy_pointer_moved(80.0, 40.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.hierarchy_pointer_size.width > 0.0);
    assert!(host.hierarchy_pointer_size.height > 0.0);
    assert_eq!(host.hierarchy_pointer_state.hovered_item_index, Some(1));
}

#[test]
fn root_hierarchy_pointer_move_prefers_shared_drawer_content_projection_over_stale_left_region_geometry()
 {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_hierarchy_content_projection_width");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    let expected_size = {
        let mut host = harness.host.borrow_mut();
        let shared_content = host
            .template_bridge
            .control_frame("LeftDrawerContentRoot")
            .expect("left drawer content root should map to a projected control frame");
        assert!(
            shared_content.width > 200.0 && shared_content.height > 100.0,
            "shared left drawer content frame should be larger than the stale fallback"
        );
        let geometry = host
            .shell_geometry
            .as_mut()
            .expect("root host should have computed shell geometry");
        let left = geometry.region_frame(ShellRegionId::Left);
        geometry.region_frames.insert(
            ShellRegionId::Left,
            ShellFrame::new(left.x, left.y, 120.0, 80.0),
        );
        UiSize::new(shared_content.width, shared_content.height)
    };

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_hierarchy_pointer_moved(80.0, 40.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(
        host.hierarchy_pointer_size, expected_size,
        "shared drawer content projection should own hierarchy pointer sizing when root callback width/height are missing"
    );
}

#[test]
fn root_console_pointer_scroll_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_console_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::BottomLeft, "editor.console#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_console_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.console_scroll_surface.size().width > 0.0);
    assert!(host.console_scroll_surface.size().height > 0.0);
}

#[test]
fn root_console_pointer_scroll_prefers_shared_drawer_content_projection_over_stale_bottom_region_geometry()
 {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_console_content_projection_height");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::BottomLeft, "editor.console#1");

    let expected_size = {
        let mut host = harness.host.borrow_mut();
        let shared_content = host
            .template_bridge
            .control_frame("BottomDrawerContentRoot")
            .expect("bottom drawer content root should map to a projected control frame");
        assert!(
            shared_content.width > 400.0 && shared_content.height > 60.0,
            "shared bottom drawer content frame should be larger than the stale fallback"
        );
        let geometry = host
            .shell_geometry
            .as_mut()
            .expect("root host should have computed shell geometry");
        let bottom = geometry.region_frame(ShellRegionId::Bottom);
        geometry.region_frames.insert(
            ShellRegionId::Bottom,
            ShellFrame::new(bottom.x, bottom.y, 260.0, 44.0),
        );
        UiSize::new(shared_content.width, shared_content.height)
    };

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_console_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(
        host.console_scroll_surface.size(),
        expected_size,
        "shared drawer content projection should own console scroll sizing when root callback width/height are missing"
    );
}

#[test]
fn root_inspector_pointer_scroll_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_inspector_projection");
    harness.activate_workbench_page();

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_inspector_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.inspector_scroll_surface.size().width > 0.0);
    assert!(host.inspector_scroll_surface.size().height > 0.0);
}

#[test]
fn root_asset_browser_details_scroll_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_asset_browser_projection");
    harness.activate_workbench_page();
    let _asset_browser = harness.open_view("editor.asset_browser");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_browser_asset_details_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.browser_asset_details_scroll_surface.size().width > 0.0);
    assert!(host.browser_asset_details_scroll_surface.size().height > 0.0);
}

#[test]
fn root_activity_asset_tree_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_activity_asset_tree_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.assets#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_tree_pointer_moved("activity".into(), 48.0, 72.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.activity_asset_pointer.tree_size.width > 0.0);
    assert!(host.activity_asset_pointer.tree_size.height > 0.0);
}

#[test]
fn root_browser_asset_tree_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_browser_asset_tree_projection");
    harness.activate_workbench_page();
    let _asset_browser = harness.open_view("editor.asset_browser");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_tree_pointer_moved("browser".into(), 48.0, 72.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.browser_asset_pointer.tree_size.width > 0.0);
    assert!(host.browser_asset_pointer.tree_size.height > 0.0);
}

#[test]
fn root_activity_asset_content_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_activity_asset_content_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.assets#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_content_pointer_moved("activity".into(), 96.0, 96.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.activity_asset_pointer.content_size.width > 0.0);
    assert!(host.activity_asset_pointer.content_size.height > 0.0);
}

#[test]
fn root_browser_asset_content_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_browser_asset_content_projection");
    harness.activate_workbench_page();
    let _asset_browser = harness.open_view("editor.asset_browser");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_content_pointer_moved("browser".into(), 96.0, 96.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.browser_asset_pointer.content_size.width > 0.0);
    assert!(host.browser_asset_pointer.content_size.height > 0.0);
}

#[test]
fn root_activity_asset_reference_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_activity_asset_reference_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.assets#1");

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_moved(
        "activity".into(),
        "references".into(),
        96.0,
        160.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    assert!(host.activity_asset_pointer.references.size.width > 0.0);
    assert!(host.activity_asset_pointer.references.size.height > 0.0);
}

#[test]
fn root_browser_asset_reference_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_browser_asset_reference_projection");
    harness.activate_workbench_page();
    let _asset_browser = harness.open_view("editor.asset_browser");

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_moved(
        "browser".into(),
        "references".into(),
        96.0,
        160.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    assert!(host.browser_asset_pointer.references.size.width > 0.0);
    assert!(host.browser_asset_pointer.references.size.height > 0.0);
}
