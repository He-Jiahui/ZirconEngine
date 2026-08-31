use super::*;

const SCOPED_PRESENTATION_UI_ASSET: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.scoped_presentation"
version = 1
display_name = "Scoped Presentation"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "Root"
props = { text = "Ready" }
"#;

#[test]
fn active_drawer_pane_presentation_uses_shell_content_without_a_full_rebuild() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_scoped_drawer_presentation");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");
    let slow_path_rebuilds_before = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    {
        let mut host = harness.host.borrow_mut();
        assert!(host.mark_presentation_dirty_for_pane("editor.hierarchy#1"));
        host.recompute_if_dirty();
    }

    assert_eq!(
        harness
            .host
            .borrow()
            .invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds_before,
        "a known active drawer pane must use the shell-content fast path"
    );
}

#[test]
fn ui_asset_mode_action_patches_the_presented_view_without_a_full_shell_rebuild() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_scoped_presentation");
    let asset_path =
        unique_temp_path("zircon_retained_scoped_presentation_asset").with_extension("ui.toml");
    std::fs::write(&asset_path, SCOPED_PRESENTATION_UI_ASSET).unwrap();
    let instance_id = harness
        .host
        .borrow()
        .editor_manager
        .open_ui_asset_editor(&asset_path, None)
        .expect("UI asset editor should open");

    {
        let mut host = harness.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    let slow_path_rebuilds_before = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_ui_asset_action(instance_id.0.as_str(), "mode.preview");
        host.recompute_if_dirty();
    }

    let presentation = harness.root_ui.get_host_presentation();
    assert_eq!(
        presentation
            .host_scene_data
            .document_dock
            .pane
            .ui_asset
            .header
            .mode,
        "Preview"
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds_before,
        "a presentation-only UI asset action must not rebuild the full shell"
    );

    let _ = std::fs::remove_file(asset_path);
}

#[test]
fn ui_asset_mode_action_patches_a_native_floating_presenter_without_a_full_shell_rebuild() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_scoped_native_presentation");
    let asset_path = unique_temp_path("zircon_retained_scoped_native_presentation_asset")
        .with_extension("ui.toml");
    std::fs::write(&asset_path, SCOPED_PRESENTATION_UI_ASSET).unwrap();
    let instance_id = harness
        .host
        .borrow()
        .editor_manager
        .open_ui_asset_editor(&asset_path, None)
        .expect("UI asset editor should open");
    {
        let mut host = harness.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), "window:ui-asset");
    let _ = child.take_external_redraw_for_test();
    let slow_path_rebuilds_before = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_ui_asset_action(instance_id.0.as_str(), "mode.preview");
        host.recompute_if_dirty();
    }

    assert_eq!(
        ui_asset_mode_in_presentation(&child.get_host_presentation(), instance_id.0.as_str()),
        Some("Preview"),
        "the native presenter must receive the scoped UI Asset pane patch"
    );
    let redraw = child.take_external_redraw_for_test();
    assert!(redraw.request_redraw());
    assert!(redraw.requires_frame_update());
    assert!(
        redraw
            .damage_region()
            .is_some_and(|damage| damage.width > 0.0 && damage.height > 0.0),
        "the native presenter patch must queue a nonempty damage region"
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds_before,
        "native presenter patching must not rebuild the full shell"
    );

    let _ = std::fs::remove_file(asset_path);
}

#[test]
fn ui_asset_scoped_patch_updates_only_its_matching_native_presenter() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_scoped_instance_isolation");
    let first_path =
        unique_temp_path("zircon_retained_scoped_instance_first").with_extension("ui.toml");
    let second_path =
        unique_temp_path("zircon_retained_scoped_instance_second").with_extension("ui.toml");
    std::fs::write(&first_path, SCOPED_PRESENTATION_UI_ASSET).unwrap();
    std::fs::write(&second_path, SCOPED_PRESENTATION_UI_ASSET).unwrap();
    let (first_instance, second_instance) = {
        let host = harness.host.borrow();
        let first = host
            .editor_manager
            .open_ui_asset_editor(&first_path, None)
            .expect("first UI asset editor should open");
        let second = host
            .editor_manager
            .open_ui_asset_editor(&second_path, None)
            .expect("second UI asset editor should open");
        (first, second)
    };
    {
        let mut host = harness.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    let first_child =
        harness.detach_view_to_child_window(first_instance.0.as_str(), "window:first");
    let second_child =
        harness.detach_view_to_child_window(second_instance.0.as_str(), "window:second");
    let slow_path_rebuilds_before = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_ui_asset_action(first_instance.0.as_str(), "mode.preview");
        host.recompute_if_dirty();
    }

    assert_eq!(
        ui_asset_mode_in_presentation(
            &harness.root_ui.get_host_presentation(),
            first_instance.0.as_str()
        ),
        Some("Preview")
    );
    assert_eq!(
        ui_asset_mode_in_presentation(
            &first_child.get_host_presentation(),
            first_instance.0.as_str()
        ),
        Some("Preview"),
        "the matching native presenter must receive the scoped patch"
    );
    assert_eq!(
        ui_asset_mode_in_presentation(
            &second_child.get_host_presentation(),
            second_instance.0.as_str()
        ),
        Some("Design"),
        "a scoped action must not rewrite a different native UI Asset instance"
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds_before,
        "the two-instance scoped path must not rebuild the full shell"
    );

    let _ = std::fs::remove_file(first_path);
    let _ = std::fs::remove_file(second_path);
}

fn ui_asset_mode_in_presentation<'a>(
    presentation: &'a crate::ui::retained_host::host_contract::HostWindowPresentationData,
    instance_id: &str,
) -> Option<&'a str> {
    let scene = &presentation.host_scene_data;
    [
        &scene.left_dock.pane,
        &scene.document_dock.pane,
        &scene.right_dock.pane,
        &scene.bottom_dock.pane,
    ]
    .into_iter()
    .chain(
        scene
            .floating_layer
            .floating_windows
            .iter()
            .map(|window| &window.active_pane),
    )
    .chain(
        presentation
            .native_floating_surface_data
            .floating_windows
            .iter()
            .map(|window| &window.active_pane),
    )
    .find(|pane| pane.kind.as_str() == "UiAssetEditor" && pane.id.as_str() == instance_id)
    .map(|pane| pane.ui_asset.header.mode.as_str())
}
