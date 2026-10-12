use super::support::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editor_event::EditorHierarchyEvent;
use crate::ui::retained_host::app::hierarchy_rename::{
    HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID, HIERARCHY_INLINE_RENAME_CONTROL_ID,
    HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX, HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID,
};
use crate::ui::retained_host::HostTextInputFocusData;

#[test]
fn failed_hierarchy_rename_commit_keeps_exact_focus_for_successful_retry() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_hierarchy_rename_failed_commit_draft");
    harness.activate_workbench_page();

    let node_id = harness
        .host
        .borrow()
        .hierarchy_scene_entries
        .iter()
        .next()
        .expect("default scene should expose a hierarchy rename target")
        .entity;
    let draft = "  Retry Rename Draft  ";
    let expected_focus = HostTextInputFocusData {
        control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
        dispatch_kind: format!("{HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX}{node_id}").into(),
        edit_action_id: HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID.into(),
        commit_action_id: HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID.into(),
        value_text: draft.into(),
        ..HostTextInputFocusData::default()
    };
    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("loaded test world should enter editor play mode");
        host.runtime.refresh_reflection();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    host_context(&harness.root_ui).set_text_input_focus(expected_focus.clone());
    assert!(host_context(&harness.root_ui).get_text_input_focus() == expected_focus);

    let failed_journal = harness.journal_len();
    let history_before_failure = history_status(&harness);
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Enter),
            PhysicalKey::Code(KeyCode::Enter),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert!(
        host_context(&harness.root_ui).get_text_input_focus() == expected_focus,
        "a synchronous Editor dispatch failure must retain the exact draft and rename target"
    );
    assert_eq!(history_status(&harness), history_before_failure);
    let failure_status = harness
        .host
        .borrow()
        .runtime
        .shell()
        .lock()
        .state
        .status_line
        .clone();
    assert!(
        failure_status.contains("scene editing is disabled during play mode"),
        "the failed synchronous rename should surface the executor error: {failure_status}"
    );
    let failed_events = harness.delta_events_since(failed_journal);
    assert!(failed_events.len() <= 1);
    assert!(failed_events.iter().all(|event| matches!(
        event,
        EditorEvent::Hierarchy(EditorHierarchyEvent::RenameNode {
            node_id: event_node,
            name: event_name,
        }) if *event_node == node_id && event_name == draft.trim()
    )));

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .shell()
            .lock()
            .state
            .exit_play_mode()
            .expect("the test should restore edit mode before retry");
        host.runtime.refresh_reflection();
    }
    assert!(
        host_context(&harness.root_ui).get_text_input_focus() == expected_focus,
        "leaving play mode must not consume the retained rename draft"
    );

    let retry_journal = harness.journal_len();
    let history_before_retry = history_status(&harness);
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Enter),
            PhysicalKey::Code(KeyCode::Enter),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert!(!host_context(&harness.root_ui)
        .get_text_input_focus()
        .is_active());
    assert_eq!(
        harness.delta_events_since(retry_journal),
        vec![EditorEvent::Hierarchy(EditorHierarchyEvent::RenameNode {
            node_id,
            name: draft.trim().to_string(),
        })]
    );
    let history_after_retry = history_status(&harness);
    assert_eq!(history_after_retry.len, history_before_retry.len + 1);
    assert!(history_after_retry.can_undo);
    assert_eq!(
        harness
            .host
            .borrow()
            .hierarchy_scene_entries
            .iter()
            .find(|entry| entry.entity == node_id)
            .map(|entry| entry.display_name.as_str()),
        Some(draft.trim())
    );
}

fn history_status(harness: &ChildWindowHostHarness) -> HistoryStatus {
    let host = harness.host.borrow();
    let shell = host.runtime.shell().lock();
    shell
        .state
        .transactions()
        .history_status(HistoryContextId::Document(
            shell.state.active_scene_document.unwrap(),
        ))
        .expect("the harness should have an active scene history")
}
