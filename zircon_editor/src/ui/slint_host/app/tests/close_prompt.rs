use super::*;
use slint::CloseRequestResponse;

const CLOSE_PROMPT_UI_ASSET: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.close_prompt"
version = 1
display_name = "Close Prompt"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "Root"
props = { text = "Ready" }
"#;

#[test]
fn dirty_floating_window_close_request_shows_cancelable_prompt() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_dirty_child_close_cancel");
    let window_id = MainPageId::new("window:assets");
    let child = harness.detach_view_to_child_window("editor.assets#1", window_id.0.as_str());
    harness
        .host
        .borrow()
        .editor_manager
        .update_view_instance_metadata(
            &ViewInstanceId::new("editor.assets#1"),
            None,
            Some(true),
            None,
        )
        .unwrap();

    let response = harness
        .host
        .borrow_mut()
        .native_floating_window_close_requested(&window_id);

    assert_eq!(response, CloseRequestResponse::KeepWindowShown);
    let prompt = child.get_host_presentation().close_prompt;
    assert!(prompt.visible);
    assert_eq!(prompt.target_window_id.as_str(), "window:assets");

    child.dispatch_native_primary_press_for_test(
        prompt.cancel_button_frame.x + 4.0,
        prompt.cancel_button_frame.y + 4.0,
    );

    assert!(!child.get_host_presentation().close_prompt.visible);
    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .floating_windows
        .iter()
        .any(|window| window.window_id == window_id));
}

#[test]
fn dirty_saveable_floating_window_save_prompt_saves_then_closes_window() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_dirty_child_close_save");
    let ui_asset_path =
        unique_temp_path("zircon_slint_close_prompt_ui_asset").with_extension("ui.toml");
    std::fs::write(&ui_asset_path, CLOSE_PROMPT_UI_ASSET).unwrap();
    let instance_id = harness
        .host
        .borrow()
        .editor_manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    {
        let mut host = harness.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    let window_id = MainPageId::new("window:ui-asset");
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), window_id.0.as_str());
    harness
        .host
        .borrow()
        .editor_manager
        .update_view_instance_metadata(&instance_id, None, Some(true), None)
        .unwrap();

    let response = harness
        .host
        .borrow_mut()
        .native_floating_window_close_requested(&window_id);

    assert_eq!(response, CloseRequestResponse::KeepWindowShown);
    let prompt = child.get_host_presentation().close_prompt;
    assert!(prompt.visible);
    assert!(prompt.can_save);

    child.dispatch_native_primary_press_for_test(
        prompt.save_button_frame.x + 4.0,
        prompt.save_button_frame.y + 4.0,
    );

    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .floating_windows
        .iter()
        .all(|window| window.window_id != window_id));
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn dirty_floating_window_discard_prompt_closes_all_window_tabs() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_dirty_child_close_discard");
    let window_id = MainPageId::new("window:assets");
    let child = harness.detach_views_to_child_window(
        &["editor.assets#1", "editor.console#1"],
        window_id.0.as_str(),
    );
    harness
        .host
        .borrow()
        .editor_manager
        .update_view_instance_metadata(
            &ViewInstanceId::new("editor.assets#1"),
            None,
            Some(true),
            None,
        )
        .unwrap();

    let response = harness
        .host
        .borrow_mut()
        .native_floating_window_close_requested(&window_id);

    assert_eq!(response, CloseRequestResponse::KeepWindowShown);
    let prompt = child.get_host_presentation().close_prompt;
    assert!(prompt.visible);

    child.dispatch_native_primary_press_for_test(
        prompt.discard_button_frame.x + 4.0,
        prompt.discard_button_frame.y + 4.0,
    );

    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .floating_windows
        .iter()
        .all(|window| window.window_id != window_id));
}

#[test]
fn dirty_main_window_discard_prompt_requests_host_exit() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_dirty_main_close_discard");
    harness
        .host
        .borrow()
        .editor_manager
        .update_view_instance_metadata(
            &ViewInstanceId::new("editor.assets#1"),
            None,
            Some(true),
            None,
        )
        .unwrap();

    let response = harness
        .host
        .borrow_mut()
        .native_main_window_close_requested();

    assert_eq!(response, CloseRequestResponse::KeepWindowShown);
    let prompt = harness.root_ui.get_host_presentation().close_prompt;
    assert!(prompt.visible);

    harness.root_ui.dispatch_native_primary_press_for_test(
        prompt.discard_button_frame.x + 4.0,
        prompt.discard_button_frame.y + 4.0,
    );

    assert!(harness.root_ui.exit_requested_for_test());
}
