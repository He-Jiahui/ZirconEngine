use super::*;
use std::time::{Duration, Instant};

use crate::core::asset::DirtyExternalEffectId;
use crate::core::extension::SaveReason;
use crate::ui::host::{DirtyDocumentSaveOwner, DirtyDocumentSaveStart};
use crate::ui::retained_host::primitives::CloseRequestResponse;

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

fn open_dirty_ui_asset(
    harness: &ChildWindowHostHarness,
    suffix: &str,
) -> (std::path::PathBuf, ViewInstanceId) {
    let path = unique_temp_path(suffix).with_extension("ui.toml");
    std::fs::write(&path, CLOSE_PROMPT_UI_ASSET).unwrap();
    let instance_id = harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .open_ui_asset_editor(&path, None)
        .expect("ui asset editor should open");
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .unwrap();
    (path, instance_id)
}

fn await_main_window_close(harness: &ChildWindowHostHarness) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !harness.root_ui.exit_requested_for_test() && Instant::now() < deadline {
        harness.host.borrow_mut().tick();
        std::thread::yield_now();
    }
    assert!(
        harness.root_ui.exit_requested_for_test(),
        "normal close must wait for real autosave, jobs and settings completion"
    );
}

fn await_owned_document_save(harness: &ChildWindowHostHarness, owner: DirtyDocumentSaveOwner) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match harness
            .host
            .borrow()
            .editor_manager
            .enter()
            .expect("editor manager admission")
            .poll_dirty_document_save(owner)
            .unwrap()
        {
            Some(_) => return,
            None if Instant::now() < deadline => std::thread::yield_now(),
            None => panic!("owned document save did not terminalize"),
        }
    }
}

#[test]
fn clean_single_tab_floating_close_uses_window_scoped_batch_command() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_clean_single_tab_close");
    let window_id = MainPageId::new("window:clean-single-tab");
    harness.detach_view_to_child_window("editor.console#1", window_id.0.as_str());
    let before_close = harness.journal_len();

    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_floating_window_close_requested(&window_id),
        CloseRequestResponse::HideWindow
    );
    assert!(!harness
        .host
        .borrow()
        .runtime
        .floating_window_exists(&window_id)
        .unwrap());
    assert!(harness
        .delta_events_since(before_close)
        .iter()
        .any(|event| matches!(
            event,
            crate::core::editor_event::EditorEvent::Layout(
                crate::core::editor_event::LayoutCommand::CloseViews { .. }
            )
        )));
}

#[test]
fn dirty_floating_window_close_request_shows_cancelable_prompt() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_child_close_cancel");
    let (ui_asset_path, instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_close_prompt_cancel");
    let window_id = MainPageId::new("window:assets");
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), window_id.0.as_str());

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
        .unwrap()
        .floating_windows
        .iter()
        .any(|window| window.window_id == window_id));
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn dirty_child_keep_shown_close_prompt_preserves_native_tab_capture() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_child_capture_prompt");
    let (ui_asset_path, instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_close_prompt_capture");
    let window_id = MainPageId::new("window:assets");
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), window_id.0.as_str());
    let presentation = child.get_host_presentation();
    let floating = presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == window_id.0.as_str())
        .expect("dirty child should expose a native floating window");
    let tab = floating
        .tab_frames
        .get(0)
        .expect("dirty child should expose a native tab");
    let x = floating.frame.x + floating.header_frame.x + tab.frame.x + 4.0;
    let y = floating.frame.y + floating.header_frame.y + tab.frame.y + tab.frame.height * 0.5;
    child.dispatch_native_primary_press_for_test(x, y);
    assert!(!host_context(&child).get_drag_state().drag_tab_id.is_empty());

    assert_eq!(
        child.dispatch_native_close_request_for_test(),
        CloseRequestResponse::KeepWindowShown
    );
    assert!(child.window().is_visible());
    assert!(child.get_host_presentation().close_prompt.visible);
    assert!(!host_context(&child).get_drag_state().drag_tab_id.is_empty());
    child.dispatch_native_focus_lost_for_test();
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn dirty_saveable_floating_window_save_prompt_schedules_then_closes_window() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_child_close_save");
    let (ui_asset_path, instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_close_prompt_ui_asset");
    {
        let mut host = harness.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    let window_id = MainPageId::new("window:ui-asset");
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), window_id.0.as_str());
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

    assert!(!child.get_host_presentation().close_prompt.can_save);
    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .unwrap()
        .floating_windows
        .iter()
        .any(|window| window.window_id == window_id));

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline
        && harness
            .host
            .borrow()
            .runtime
            .current_layout()
            .unwrap()
            .floating_windows
            .iter()
            .any(|window| window.window_id == window_id)
    {
        harness.host.borrow_mut().tick();
        std::thread::yield_now();
    }

    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .unwrap()
        .floating_windows
        .iter()
        .all(|window| window.window_id != window_id));
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn dirty_floating_window_discard_prompt_closes_all_window_tabs() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_child_close_discard");
    let (ui_asset_path, instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_close_prompt_discard");
    let window_id = MainPageId::new("window:assets");
    let child = harness.detach_views_to_child_window(
        &[instance_id.0.as_str(), "editor.console#1"],
        window_id.0.as_str(),
    );

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
        .unwrap()
        .floating_windows
        .iter()
        .all(|window| window.window_id != window_id));
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn floating_discard_reprompts_when_dirty_generation_changes_before_decision() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_child_close_stale_discard");
    let (ui_asset_path, instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_close_prompt_stale_discard");
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .update_ui_asset_editor_source(
            &instance_id,
            CLOSE_PROMPT_UI_ASSET.replace("Ready", "First"),
        )
        .unwrap();
    let window_id = MainPageId::new("window:assets");
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), window_id.0.as_str());
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_floating_window_close_requested(&window_id),
        CloseRequestResponse::KeepWindowShown
    );
    let old_prompt = child.get_host_presentation().close_prompt;
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .update_ui_asset_editor_source(
            &instance_id,
            CLOSE_PROMPT_UI_ASSET.replace("Ready", "Second"),
        )
        .unwrap();
    child.dispatch_native_primary_press_for_test(
        old_prompt.discard_button_frame.x + 4.0,
        old_prompt.discard_button_frame.y + 4.0,
    );
    assert!(child.get_host_presentation().close_prompt.visible);
    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .unwrap()
        .floating_windows
        .iter()
        .any(|window| window.window_id == window_id));
    let refreshed_prompt = child.get_host_presentation().close_prompt;
    let before_discard = harness.journal_len();
    child.dispatch_native_primary_press_for_test(
        refreshed_prompt.discard_button_frame.x + 4.0,
        refreshed_prompt.discard_button_frame.y + 4.0,
    );
    assert!(harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .unwrap()
        .floating_windows
        .iter()
        .all(|window| window.window_id != window_id));
    assert!(harness
        .delta_events_since(before_discard)
        .iter()
        .any(|event| matches!(
            event,
            crate::core::editor_event::EditorEvent::Layout(
                crate::core::editor_event::LayoutCommand::CloseViews { .. }
            )
        )));
    let host = harness.host.borrow();
    let journal = host.runtime.journal();
    let close_record = journal.records()[before_discard..]
        .iter()
        .find(|record| {
            matches!(
                &record.event,
                crate::core::editor_event::EditorEvent::Layout(
                    crate::core::editor_event::LayoutCommand::CloseViews { .. }
                )
            )
        })
        .expect("authorized close must publish a durable layout record");
    assert!(close_record
        .effects
        .contains(&crate::core::editor_event::EditorEventEffect::LayoutChanged));
    assert!(close_record
        .effects
        .contains(&crate::core::editor_event::EditorEventEffect::ReflectionChanged));
    drop(host);
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn floating_discard_reprompts_if_a_tab_is_attached_after_the_decision() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_close_late_tab");
    let (ui_asset_path, instance_id) = open_dirty_ui_asset(&harness, "zircon_close_late_tab");
    let window_id = MainPageId::new("window:late-tab");
    let child = harness.detach_view_to_child_window(instance_id.0.as_str(), window_id.0.as_str());
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_floating_window_close_requested(&window_id),
        CloseRequestResponse::KeepWindowShown
    );
    let old_prompt = child.get_host_presentation().close_prompt;
    harness.detach_views_to_child_window(&["editor.console#1"], window_id.0.as_str());
    child.dispatch_native_primary_press_for_test(
        old_prompt.discard_button_frame.x + 4.0,
        old_prompt.discard_button_frame.y + 4.0,
    );
    let refreshed = child.get_host_presentation().close_prompt;
    assert!(refreshed.visible);
    let window = harness
        .host
        .borrow()
        .runtime
        .current_layout()
        .unwrap()
        .floating_windows
        .into_iter()
        .find(|window| window.window_id == window_id)
        .unwrap();
    let mut current = Vec::new();
    window.workspace.append_instance_ids(&mut current);
    assert!(current.contains(&instance_id));
    assert!(current.contains(&ViewInstanceId::new("editor.console#1")));
    child.dispatch_native_primary_press_for_test(
        refreshed.cancel_button_frame.x + 4.0,
        refreshed.cancel_button_frame.y + 4.0,
    );
    assert!(harness
        .host
        .borrow()
        .runtime
        .floating_window_exists(&window_id)
        .unwrap());
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn dirty_main_window_discard_prompt_requests_host_exit() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_main_close_discard");
    let (ui_asset_path, _instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_close_prompt_main_discard");

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

    assert!(!harness.root_ui.exit_requested_for_test());
    await_main_window_close(&harness);
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn dirty_project_close_defers_teardown_until_the_shared_discard_decision() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_dirty_project_close");
    let (ui_asset_path, _instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_project_close_prompt");

    harness.dispatch_menu_action("workbench.project.close");

    let prompt = harness.root_ui.get_host_presentation().close_prompt;
    assert!(prompt.visible);
    assert_eq!(prompt.target_window_id.as_str(), "main");
    assert_eq!(
        prompt.title.as_str(),
        "Save changes before closing project?"
    );
    assert_eq!(
        harness.host.borrow().startup_session.mode,
        crate::ui::workbench::startup::EditorSessionMode::Project
    );

    harness.root_ui.dispatch_native_primary_press_for_test(
        prompt.cancel_button_frame.x + 4.0,
        prompt.cancel_button_frame.y + 4.0,
    );
    assert_eq!(
        harness.host.borrow().startup_session.mode,
        crate::ui::workbench::startup::EditorSessionMode::Project
    );

    harness.dispatch_menu_action("workbench.project.close");
    let prompt = harness.root_ui.get_host_presentation().close_prompt;
    harness.root_ui.dispatch_native_primary_press_for_test(
        prompt.discard_button_frame.x + 4.0,
        prompt.discard_button_frame.y + 4.0,
    );

    assert_eq!(
        harness.host.borrow().startup_session.mode,
        crate::ui::workbench::startup::EditorSessionMode::Welcome
    );
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn save_all_queues_behind_close_prompt_save_then_acquires_the_released_owner() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_save_all_owner_queue");
    let (ui_asset_path, instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_save_all_owner_queue");
    let document = harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .dirty_document_toolkits()
        .unwrap()[0]
        .document_id;

    assert_eq!(
        crate::ui::host::EditorManager::begin_dirty_document_save(
            harness.host.borrow().editor_manager.clone(),
            DirtyDocumentSaveOwner::ClosePrompt,
            [document],
            SaveReason::Explicit,
        )
        .unwrap(),
        DirtyDocumentSaveStart::Scheduled
    );
    harness.host.borrow_mut().request_document_save_all();
    assert!(harness.host.borrow().queued_document_save_all);
    assert!(!harness.host.borrow().pending_document_save_all);

    await_owned_document_save(&harness, DirtyDocumentSaveOwner::ClosePrompt);
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .unwrap();
    harness.host.borrow_mut().poll_document_save_all();
    assert!(!harness.host.borrow().queued_document_save_all);
    assert!(harness.host.borrow().pending_document_save_all);

    let deadline = Instant::now() + Duration::from_secs(2);
    while harness.host.borrow().pending_document_save_all && Instant::now() < deadline {
        harness.host.borrow_mut().poll_document_save_all();
        std::thread::yield_now();
    }
    assert!(!harness.host.borrow().pending_document_save_all);
    assert!(harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .dirty_document_toolkits()
        .unwrap()
        .is_empty());
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn project_close_cannot_teardown_until_close_prompt_save_terminalizes() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_project_close_save_owner");
    let (ui_asset_path, _instance_id) =
        open_dirty_ui_asset(&harness, "zircon_retained_project_close_save_owner");
    let document = harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .expect("editor manager admission")
        .dirty_document_toolkits()
        .unwrap()[0]
        .document_id;
    assert_eq!(
        crate::ui::host::EditorManager::begin_dirty_document_save(
            harness.host.borrow().editor_manager.clone(),
            DirtyDocumentSaveOwner::ClosePrompt,
            [document],
            SaveReason::Explicit,
        )
        .unwrap(),
        DirtyDocumentSaveStart::Scheduled
    );

    harness.host.borrow_mut().request_project_close().unwrap();
    assert_eq!(
        harness.host.borrow().startup_session.mode,
        crate::ui::workbench::startup::EditorSessionMode::Project
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .editor_manager
            .enter()
            .expect("editor manager admission")
            .dirty_document_save_owner(),
        Some(DirtyDocumentSaveOwner::ClosePrompt)
    );

    await_owned_document_save(&harness, DirtyDocumentSaveOwner::ClosePrompt);
    harness.host.borrow_mut().request_project_close().unwrap();
    assert_eq!(
        harness.host.borrow().startup_session.mode,
        crate::ui::workbench::startup::EditorSessionMode::Welcome
    );
    let _ = std::fs::remove_file(ui_asset_path);
}

#[test]
fn clean_child_hide_close_cancels_native_tab_without_detaching_it() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_clean_child_capture_close");
    let window_id = MainPageId::new("window:clean-capture");
    let child = harness.detach_view_to_child_window("editor.console#1", window_id.0.as_str());
    let presentation = child.get_host_presentation();
    let floating = presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == window_id.0.as_str())
        .expect("clean child should expose a native floating window");
    let tab = floating
        .tab_frames
        .get(0)
        .expect("clean child should expose a native tab");
    let x = floating.frame.x + floating.header_frame.x + tab.frame.x + 4.0;
    let y = floating.frame.y + floating.header_frame.y + tab.frame.y + tab.frame.height * 0.5;
    child.dispatch_native_primary_press_for_test(x, y);
    child.dispatch_native_pointer_move_for_test(x + 40.0, y + 30.0);
    assert!(!host_context(&child).get_drag_state().drag_tab_id.is_empty());
    let baseline = harness.journal_len();

    assert_eq!(
        child.dispatch_native_close_request_for_test(),
        CloseRequestResponse::HideWindow
    );
    assert!(!child.window().is_visible());
    assert!(host_context(&child).get_drag_state().drag_tab_id.is_empty());
    assert!(!harness
        .host
        .borrow()
        .runtime
        .floating_window_exists(&window_id)
        .unwrap());
    let close_events = harness.delta_events_since(baseline);
    assert!(close_events.iter().any(|event| matches!(
        event,
        crate::core::editor_event::EditorEvent::Layout(
            crate::core::editor_event::LayoutCommand::CloseViews { .. }
        )
    )));
    assert!(!close_events.iter().any(|event| matches!(
        event,
        crate::core::editor_event::EditorEvent::Layout(
            crate::core::editor_event::LayoutCommand::DetachViewToWindow { .. }
        )
    )));
    let after_close = harness.journal_len();
    child.dispatch_native_primary_release_for_test(x + 40.0, y + 30.0);
    assert_eq!(harness.journal_len(), after_close, "late Up cannot drop");
}

#[test]
fn programmatic_child_hide_retires_only_its_native_tab_capture() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_programmatic_child_hide_capture");
    let hidden_id = MainPageId::new("window:hidden-capture");
    let other_id = MainPageId::new("window:other-capture");
    let hidden = harness.detach_view_to_child_window("editor.console#1", hidden_id.0.as_str());
    let other = harness.detach_view_to_child_window("editor.hierarchy#1", other_id.0.as_str());

    let hidden_window = hidden
        .get_host_presentation()
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == hidden_id.0.as_str())
        .cloned()
        .expect("first child should expose its floating tab");
    let hidden_tab = hidden_window
        .tab_frames
        .get(0)
        .expect("first child should expose a native tab");
    let hidden_x = hidden_window.frame.x + hidden_window.header_frame.x + hidden_tab.frame.x + 4.0;
    let hidden_y = hidden_window.frame.y
        + hidden_window.header_frame.y
        + hidden_tab.frame.y
        + hidden_tab.frame.height * 0.5;
    hidden.dispatch_native_primary_press_for_test(hidden_x, hidden_y);
    hidden.dispatch_native_pointer_move_for_test(hidden_x + 40.0, hidden_y + 30.0);
    host_context(&other).set_drag_state(crate::ui::retained_host::HostDragStateData {
        drag_tab_id: "other-tab-capture".to_owned(),
        ..Default::default()
    });
    assert!(!host_context(&hidden)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert!(!host_context(&other).get_drag_state().drag_tab_id.is_empty());
    let journal_before_hide = harness.journal_len();

    // Hide runs while the app host is mutably borrowed. It must retire this native state without
    // calling back into the app host.
    let host = harness.host.borrow_mut();
    hidden
        .hide()
        .expect("programmatic child hide should succeed");
    drop(host);

    assert!(!hidden.window().is_visible());
    assert!(host_context(&hidden)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert!(!host_context(&other).get_drag_state().drag_tab_id.is_empty());
    hidden.dispatch_native_primary_release_for_test(hidden_x + 40.0, hidden_y + 30.0);
    assert_eq!(
        harness.journal_len(),
        journal_before_hide,
        "late Up is inert"
    );

    host_context(&other).set_drag_state(Default::default());
    hidden.show().expect("hidden child should be reusable");
    hidden.dispatch_native_primary_press_for_test(hidden_x, hidden_y);
    hidden.dispatch_native_pointer_move_for_test(hidden_x + 40.0, hidden_y + 30.0);
    assert!(!host_context(&hidden)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert!(host_context(&other).get_drag_state().drag_tab_id.is_empty());
}

struct CloseProducerAfterFence {
    entered: std::sync::mpsc::Sender<()>,
    release: std::sync::mpsc::Receiver<()>,
    edit: Box<dyn FnOnce() -> (bool, bool) + Send>,
}

impl crate::core::jobs::EditorJob for CloseProducerAfterFence {
    type Output = (bool, bool);

    fn run(
        self,
        _context: crate::core::jobs::JobContext,
    ) -> Result<Self::Output, crate::core::jobs::JobError> {
        self.entered
            .send(())
            .map_err(crate::core::jobs::JobError::failed)?;
        self.release
            .recv()
            .map_err(crate::core::jobs::JobError::failed)?;
        Ok((self.edit)())
    }
}

#[test]
fn main_window_close_rejects_late_scene_and_document_writes_until_real_job_terminal() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_owned_close_late_write");
    let (path, instance) = open_dirty_ui_asset(&harness, "zircon_owned_close_late_ui");
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let context = harness.host.borrow().runtime.context().clone();
    let manager = harness.host.borrow().editor_manager.clone();
    let worker_context = context.clone();
    let ticket = context
        .jobs()
        .submit(
            crate::core::jobs::EditorJobSpec::new(
                "Late close producer",
                crate::core::jobs::JobCategory::Misc,
            ),
            CloseProducerAfterFence {
                entered: entered_tx,
                release: release_rx,
                edit: Box::new(move || {
                    let scene_rejected = worker_context
                        .transactions()
                        .begin_exclusive_transition("late close producer")
                        .is_err();
                    let document_rejected = manager
                        .enter()
                        .expect("live manager during close")
                        .update_ui_asset_editor_source(&instance, CLOSE_PROMPT_UI_ASSET.to_string())
                        .is_err();
                    (scene_rejected, document_rejected)
                }),
            },
        )
        .expect("producer admitted before close");
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("real producer started");

    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_main_window_close_requested(),
        CloseRequestResponse::KeepWindowShown
    );
    let prompt = harness.root_ui.get_host_presentation().close_prompt;
    assert!(prompt.visible);
    harness.root_ui.dispatch_native_primary_press_for_test(
        prompt.discard_button_frame.x + 4.0,
        prompt.discard_button_frame.y + 4.0,
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    while context.jobs().shutdown_progress().state()
        == crate::core::jobs::EditorJobShutdownState::NotRequested
        && Instant::now() < deadline
    {
        harness.host.borrow_mut().tick();
        std::thread::yield_now();
    }
    assert_eq!(
        context.jobs().shutdown_progress().state(),
        crate::core::jobs::EditorJobShutdownState::Pending
    );
    assert!(harness.host.borrow().main_window_close_blocks_session_ui());
    assert!(!harness.root_ui.exit_requested_for_test());
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_main_window_close_requested(),
        CloseRequestResponse::KeepWindowShown
    );
    assert!(!harness.root_ui.exit_requested_for_test());

    release_tx.send(()).unwrap();
    await_main_window_close(&harness);
    assert_eq!(ticket.try_take_terminal(), Some(Ok((true, true))));
    assert_eq!(
        context.jobs().shutdown_progress().state(),
        crate::core::jobs::EditorJobShutdownState::Quiescent
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn main_window_close_keeps_failed_settings_owner_and_idle_wake_after_jobs_quiesce() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_owned_close_settings_failure");
    let context = harness.host.borrow().runtime.context().clone();
    let already_closed = context.settings_mutations().shutdown();
    assert!(matches!(already_closed.poll_result(), Some(Ok(_))));
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_main_window_close_requested(),
        CloseRequestResponse::KeepWindowShown
    );
    for _ in 0..8 {
        harness.host.borrow_mut().tick();
    }
    assert_eq!(
        context.jobs().shutdown_progress().state(),
        crate::core::jobs::EditorJobShutdownState::Quiescent
    );
    assert!(!harness.root_ui.exit_requested_for_test());
    assert!(harness.host.borrow().main_window_close_blocks_session_ui());
    assert!(harness
        .root_ui
        .lifecycle_frame_wake_deadline_for_test()
        .is_some());
    let status = harness.host.borrow().runtime.status_line();
    assert!(!status.is_empty());
    for _ in 0..3 {
        assert_eq!(
            harness
                .host
                .borrow_mut()
                .native_main_window_close_requested(),
            CloseRequestResponse::KeepWindowShown
        );
        harness.host.borrow_mut().tick();
    }
    assert_eq!(harness.host.borrow().runtime.status_line(), status);
    assert!(!harness.root_ui.exit_requested_for_test());
    assert!(context
        .transactions()
        .begin_exclusive_transition("failed close late edit")
        .is_err());
}

fn editor09_real_close_session(suffix: &str) -> (std::path::PathBuf, ChildWindowHostHarness) {
    use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};

    let configured = std::env::var("CARGO_TARGET_DIR")
        .expect("Editor09 fixtures require a preregistered physical E: target root");
    assert!(configured
        .replace('\\', "/")
        .trim_end_matches('/')
        .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"));
    let target = std::path::PathBuf::from(configured).canonicalize().unwrap();
    let physical = target.to_string_lossy().replace('\\', "/");
    let physical = physical.strip_prefix("//?/").unwrap_or(&physical);
    assert!(physical.eq_ignore_ascii_case("E:/cargo-targets/zircon-local"));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output = target.join("zircon-test-output").join(format!(
        "editor09-final-close-p1-{suffix}-{}-{nonce}",
        std::process::id()
    ));
    assert!(!output.exists());
    std::fs::create_dir_all(&output).unwrap();
    // The native owner preregisters this output family. Failed cases retain it.
    eprintln!("EDITOR09_CLOSE_P1_FIXTURE={}", output.display());
    let created = ProjectAuthority::default()
        .create_project(
            &NewProjectDraft {
                project_name: "CloseSession".into(),
                location: output.to_string_lossy().into_owned(),
                template: ProjectTemplateId::RenderableEmpty,
            },
            &crate::tests::support::test_project_creation_provenance(),
        )
        .expect("create the same real project used by existing manager-open tests");
    let harness = ChildWindowHostHarness::new_at_config_path(output.join("config.json"));
    harness.activate_scene_for_editor09_close_fixture();
    let level = {
        let host = harness.host.borrow();
        let manager = host.editor_manager.enter().expect("real manager admission");
        let document = manager
            .open_project(&created.root)
            .expect("real Session admission");
        assert!(manager.project_close_activation_identity().is_some());
        assert!(manager.project_session_heartbeat_deadline().is_some());
        manager.prepare_authoring_world(document.world).unwrap()
    };
    harness
        .host
        .borrow()
        .runtime
        .replace_world(level, created.root.to_string_lossy())
        .unwrap();
    if harness.host.borrow().project_close_coordinator.phase()
        == crate::ui::host::ProjectCloseCoordinatorPhase::Closed
    {
        harness
            .host
            .borrow_mut()
            .project_close_coordinator
            .reset_for_new_session()
            .unwrap();
    }
    assert_eq!(
        harness.host.borrow().project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Decision
    );
    (output, harness)
}

fn editor09_open_session_dirty_ui(
    harness: &ChildWindowHostHarness,
    output: &std::path::Path,
) -> ViewInstanceId {
    let path = {
        let host = harness.host.borrow();
        let manager = host.editor_manager.enter().expect("real manager admission");
        let activation = manager.project_close_activation_identity().unwrap();
        let project_root = activation.project_root().canonicalize().unwrap();
        assert!(project_root.starts_with(output));
        project_root.join("Assets").join("close.ui.toml")
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, CLOSE_PROMPT_UI_ASSET).unwrap();
    let host = harness.host.borrow();
    let manager = host.editor_manager.enter().expect("real manager admission");
    let instance = manager.open_ui_asset_editor(&path, None).unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance,
            CLOSE_PROMPT_UI_ASSET.replace("Ready", "Approved"),
        )
        .unwrap();
    instance
}

fn editor09_approve_main_window_discard(harness: &ChildWindowHostHarness) {
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_main_window_close_requested(),
        CloseRequestResponse::KeepWindowShown
    );
    let prompt = harness.root_ui.get_host_presentation().close_prompt;
    assert!(prompt.visible);
    harness.root_ui.dispatch_native_primary_press_for_test(
        prompt.discard_button_frame.x + 4.0,
        prompt.discard_button_frame.y + 4.0,
    );
    assert!(harness.host.borrow().main_window_close_requested());
    assert!(!harness.root_ui.exit_requested_for_test());
}

#[test]
fn main_window_stale_decision_cancel_retires_owner_before_real_session_closing() {
    let _guard = lock_env();
    let (output, harness) = editor09_real_close_session("cancel");
    let instance = editor09_open_session_dirty_ui(&harness, &output);
    let (activation, heartbeat) = {
        let host = harness.host.borrow();
        let manager = host.editor_manager.enter().unwrap();
        (
            manager.project_close_activation_identity().unwrap(),
            manager.project_session_heartbeat_deadline().unwrap(),
        )
    };
    editor09_approve_main_window_discard(&harness);
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .update_ui_asset_editor_source(
            &instance,
            CLOSE_PROMPT_UI_ASSET.replace("Ready", "Late before close"),
        )
        .unwrap();
    harness.host.borrow_mut().poll_owned_main_window_close();
    assert!(harness.host.borrow().main_window_close_waiting_decision());
    assert_eq!(
        harness.host.borrow().project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Decision
    );
    let refreshed = harness.root_ui.get_host_presentation().close_prompt;
    assert!(refreshed.visible);
    harness.root_ui.dispatch_native_primary_press_for_test(
        refreshed.cancel_button_frame.x + 4.0,
        refreshed.cancel_button_frame.y + 4.0,
    );
    {
        let host = harness.host.borrow();
        assert!(!host.main_window_close_requested());
        assert!(host.main_window_close_poll_deadline().is_none());
        assert!(host.pending_close_prompt.is_none());
        assert!(!host.main_window_close_blocks_session_ui());
        let manager = host.editor_manager.enter().unwrap();
        assert_eq!(
            manager.project_close_activation_identity(),
            Some(activation)
        );
        assert_eq!(
            manager.project_session_heartbeat_deadline(),
            Some(heartbeat)
        );
        manager
            .update_ui_asset_editor_source(
                &instance,
                CLOSE_PROMPT_UI_ASSET.replace("Ready", "Edit after Cancel"),
            )
            .unwrap();
        let context = host.runtime.context();
        let mutation = context
            .transactions()
            .begin_exclusive_transition("edit after Cancel")
            .expect("Cancel before Closing preserves ordinary Scene mutation admission");
        drop(mutation);
    }
    assert!(!harness.root_ui.exit_requested_for_test());
    drop(harness);
    std::fs::remove_dir_all(output).unwrap();
}

#[test]
fn main_window_quiesced_late_revision_retains_real_session_and_mutation_caps() {
    let _guard = lock_env();
    let (output, harness) = editor09_real_close_session("recovery");
    let instance = editor09_open_session_dirty_ui(&harness, &output);
    let activation = harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .project_close_activation_identity()
        .unwrap();
    editor09_approve_main_window_discard(&harness);
    // The actual coordinator can own Closing before the outer close caller has
    // stored the return value. Exercise that ownership boundary with real effects.
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .quiesce_project_close_for_main_window(Some(activation.clone()))
            .unwrap(),
        Some(activation.clone())
    );
    assert_eq!(
        harness.host.borrow().project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Quiescing
    );
    assert!(harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .project_session_heartbeat_deadline()
        .is_none());
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .update_ui_asset_editor_source(
            &instance,
            CLOSE_PROMPT_UI_ASSET.replace("Ready", "Late after close effects"),
        )
        .unwrap();
    harness.host.borrow_mut().poll_owned_main_window_close();
    {
        let mut host = harness.host.borrow_mut();
        assert!(!host.main_window_close_waiting_decision());
        assert!(host.main_window_close_blocks_session_ui());
        assert_eq!(
            host.project_close_coordinator.phase(),
            crate::ui::host::ProjectCloseCoordinatorPhase::RecoveryRequired
        );
        assert!(!host.cancel_owned_main_window_close_decision());
        assert!(host.main_window_close_requested());
        assert!(host.main_window_close_poll_deadline().is_some());
        let manager = host.editor_manager.enter().unwrap();
        assert_eq!(
            manager.project_close_activation_identity(),
            Some(activation)
        );
        assert!(manager.project_session_heartbeat_deadline().is_none());
        assert!(manager
            .update_ui_asset_editor_source(
                &instance,
                CLOSE_PROMPT_UI_ASSET.replace("Ready", "Rejected late edit")
            )
            .is_err());
        assert!(host
            .runtime
            .context()
            .transactions()
            .begin_exclusive_transition("late edit after Closing")
            .is_err());
    }
    assert!(!harness.root_ui.exit_requested_for_test());
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_main_window_close_requested(),
        CloseRequestResponse::KeepWindowShown
    );
    drop(harness);
    std::fs::remove_dir_all(output).unwrap();
}

struct Editor09PendingAssetDeletion<T> {
    entered: std::sync::mpsc::Sender<()>,
    release: std::sync::mpsc::Receiver<()>,
    output: std::marker::PhantomData<fn() -> T>,
}

impl<T: Send + 'static> crate::core::jobs::EditorJob for Editor09PendingAssetDeletion<T> {
    type Output = T;

    fn run(
        self,
        _context: crate::core::jobs::JobContext,
    ) -> Result<T, crate::core::jobs::JobError> {
        self.entered
            .send(())
            .map_err(crate::core::jobs::JobError::failed)?;
        self.release
            .recv()
            .map_err(crate::core::jobs::JobError::failed)?;
        Err(crate::core::jobs::JobError::Cancelled)
    }
}

#[test]
fn main_window_pending_asset_quiesce_owns_closing_before_cancel_can_run() {
    let _guard = lock_env();
    let (output, harness) = editor09_real_close_session("pending");
    let instance = editor09_open_session_dirty_ui(&harness, &output);
    let activation = harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .project_close_activation_identity()
        .unwrap();
    let context = harness.host.borrow().runtime.context().clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let ticket = context
        .jobs()
        .submit(
            crate::core::jobs::EditorJobSpec::new(
                "Editor09 real pending asset close",
                crate::core::jobs::JobCategory::Misc,
            ),
            Editor09PendingAssetDeletion {
                entered: entered_tx,
                release: release_rx,
                output: std::marker::PhantomData,
            },
        )
        .expect("admit a real runtime-owned task before closing");
    entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("real asset task entered");
    let deletion = crate::core::asset::EditorAssetDeletionTicket::new(ticket, Default::default());
    harness
        .host
        .borrow_mut()
        .install_pending_asset_deletion_for_editor09_close_test(deletion);
    editor09_approve_main_window_discard(&harness);

    // This real pending asset task makes quiesce return before its outer Ok
    // assignment, after the Session and coordinator already entered Closing.
    harness.host.borrow_mut().poll_owned_main_window_close();
    assert_eq!(
        harness.host.borrow().project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Quiescing
    );
    assert!(harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .project_session_heartbeat_deadline()
        .is_none());
    assert!(!harness.host.borrow().main_window_close_waiting_decision());
    assert!(!harness.root_ui.exit_requested_for_test());
    harness
        .host
        .borrow()
        .editor_manager
        .enter()
        .unwrap()
        .update_ui_asset_editor_source(
            &instance,
            CLOSE_PROMPT_UI_ASSET.replace("Ready", "Late while asset quiesce is pending"),
        )
        .unwrap();
    harness.host.borrow_mut().poll_owned_main_window_close();
    assert!(!harness.host.borrow().main_window_close_waiting_decision());
    assert_eq!(
        harness.host.borrow().project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Quiescing
    );
    assert!(!harness.root_ui.exit_requested_for_test());

    release_tx.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        context.jobs().pump_events();
        let progress = context.jobs().shutdown_progress();
        if progress.unfinished_jobs().is_empty()
            && !progress.completion_callbacks_pending()
            && !progress.owned_tasks_pending()
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the real runtime handle and its observers must terminalize before fixture cleanup"
        );
        std::thread::yield_now();
    }
    // Result availability alone was insufficient; the preceding loop observes
    // the actual runtime handle registry and completion barrier.
    {
        let mut host = harness.host.borrow_mut();
        assert!(matches!(
            host.commit_project_close(),
            Err(super::super::project_close::RetainedProjectCloseError::MainWindowCloseOwned)
        ));
        // Force the same-tick interleaving: owned quiesce saw Pending, then the
        // ordinary deletion poll sees the result. It may consume that ticket,
        // but it cannot commit or release the main-window-owned Session.
        host.poll_asset_deletion();
        assert_eq!(
            host.project_close_coordinator.phase(),
            crate::ui::host::ProjectCloseCoordinatorPhase::Quiescing
        );
        let manager = host.editor_manager.enter().unwrap();
        assert_eq!(
            manager.project_close_activation_identity(),
            Some(activation.clone())
        );
        assert!(manager.project_session_heartbeat_deadline().is_none());
    }
    assert!(!harness.root_ui.exit_requested_for_test());
    // Resume the real owned quiesce path and revalidate after acquiring caps.
    harness.host.borrow_mut().poll_owned_main_window_close();
    {
        let mut host = harness.host.borrow_mut();
        assert!(!host.main_window_close_waiting_decision());
        assert_eq!(
            host.project_close_coordinator.phase(),
            crate::ui::host::ProjectCloseCoordinatorPhase::RecoveryRequired
        );
        assert!(!host.cancel_owned_main_window_close_decision());
        assert!(host.main_window_close_requested());
        assert!(host.main_window_close_blocks_session_ui());
        assert!(host.main_window_close_poll_deadline().is_some());
        let manager = host.editor_manager.enter().unwrap();
        assert_eq!(
            manager.project_close_activation_identity(),
            Some(activation)
        );
        assert!(manager.project_session_heartbeat_deadline().is_none());
        assert!(manager
            .update_ui_asset_editor_source(
                &instance,
                CLOSE_PROMPT_UI_ASSET.replace("Ready", "Rejected after pending retry")
            )
            .is_err());
        assert!(host
            .runtime
            .context()
            .transactions()
            .begin_exclusive_transition("late edit after pending close")
            .is_err());
    }
    assert!(!harness.root_ui.exit_requested_for_test());
    drop(context);
    drop(harness);
    std::fs::remove_dir_all(output).unwrap();
}
