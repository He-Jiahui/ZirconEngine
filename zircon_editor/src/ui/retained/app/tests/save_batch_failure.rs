use super::super::close_prompt::ClosePromptTarget;
use super::support::{lock_env, ChildWindowHostHarness};
use crate::ui::host::{register_completion_toolkit, CompletionAction};
use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::layout::MainPageId;

fn actual_committed_projection_failure_blocks_close(floating: bool) {
    let _environment = lock_env();
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("retained_batch_failure").unwrap();
    let config = fixture.create_file("config.json", b"{}").unwrap();
    let harness = ChildWindowHostHarness::new_at_config_path(config);
    let original = harness.host.borrow().editor_manager.clone();
    let (document, instance, _, source_path) = {
        let manager = original.enter().unwrap();
        register_completion_toolkit(&manager, &mut fixture, CompletionAction::RemoveView)
    };
    {
        let mut host = harness.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    let window_id = MainPageId::new("window:failed-batch-close");
    let window = if floating {
        harness.detach_views_to_child_window(
            &[instance.0.as_str(), "editor.console#1"],
            window_id.0.as_str(),
        )
    } else {
        harness.root_ui.clone()
    };
    let expected_target = if floating {
        ClosePromptTarget::FloatingWindow(window_id.clone())
    } else {
        ClosePromptTarget::MainWindow
    };
    assert_eq!(
        window.dispatch_native_close_request_for_test(),
        CloseRequestResponse::KeepWindowShown
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .pending_close_prompt
            .as_ref()
            .unwrap()
            .target,
        expected_target
    );
    harness
        .host
        .borrow_mut()
        .close_prompt_action_clicked("save");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while harness
        .host
        .borrow()
        .pending_close_prompt
        .as_ref()
        .is_some_and(|prompt| prompt.save_in_flight())
    {
        assert!(
            std::time::Instant::now() < deadline,
            "real close-prompt batch did not terminalize"
        );
        harness.host.borrow_mut().poll_prompted_close_save();
        std::thread::yield_now();
    }
    let failed_decision = {
        let host = harness.host.borrow();
        let prompt = host
            .pending_close_prompt
            .as_ref()
            .expect("failed actual projection must not request exit");
        assert!(prompt.has_failed_save());
        assert!(
            prompt.dirty_views.is_empty(),
            "the actual lower completion cleared Dirty before projection failed"
        );
        assert!(!prompt.save_in_flight());
        prompt.clone()
    };
    assert!(!original
        .enter()
        .unwrap()
        .context()
        .dirty_documents()
        .snapshot(document)
        .unwrap()
        .is_dirty());
    assert!(std::fs::read(&source_path).unwrap().is_empty());
    assert_eq!(
        window.dispatch_native_close_request_for_test(),
        CloseRequestResponse::KeepWindowShown
    );
    assert_eq!(harness.host.borrow().pending_close_prompt.as_ref(), Some(&failed_decision),
        "a second real native close cannot hide/close the window or replace the original failed decision");
    harness
        .host
        .borrow_mut()
        .close_prompt_action_clicked("save");
    {
        let host = harness.host.borrow();
        let prompt = host
            .pending_close_prompt
            .as_ref()
            .expect("NoDirtyDocuments cannot dismiss a previous failure");
        assert!(prompt.has_failed_save());
        assert_eq!(prompt.target, expected_target);
        assert!(prompt.dirty_views.is_empty());
        assert!(!prompt.save_in_flight());
        assert!(host
            .runtime
            .editor_snapshot()
            .unwrap()
            .status_line
            .contains("previous save failed"));
    }
    assert_eq!(
        window.dispatch_native_close_request_for_test(),
        CloseRequestResponse::KeepWindowShown
    );
    assert!(harness
        .host
        .borrow()
        .pending_close_prompt
        .as_ref()
        .unwrap()
        .has_failed_save());
    assert!(!harness.root_ui.exit_requested_for_test());
    if floating {
        assert!(harness
            .host
            .borrow()
            .runtime
            .floating_window_exists(&window_id)
            .unwrap());
    }
    harness
        .host
        .borrow_mut()
        .close_prompt_action_clicked("cancel");
    assert!(
        harness.host.borrow().pending_close_prompt.is_none(),
        "explicit Cancel retires the failed decision"
    );
    drop(window);
    drop(harness);
    fixture.expect_written_bytes("document.zdoc", b"").unwrap();
}

#[test]
fn actual_failed_batch_blocks_repeated_main_close_and_repeated_save() {
    actual_committed_projection_failure_blocks_close(false);
}

#[test]
fn actual_failed_batch_blocks_repeated_floating_close_and_repeated_save() {
    actual_committed_projection_failure_blocks_close(true);
}
