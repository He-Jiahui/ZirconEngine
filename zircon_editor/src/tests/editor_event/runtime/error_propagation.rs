use super::*;

use std::sync::Arc;

use crate::core::editing::operation::{
    DeferredOperationInvocation, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, OperationCommandFactoryRegistration, PendingEditRetention,
};
use crate::core::editor_event::SelectionHostEvent;
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::editing::operation::EditOperationTarget;
use crate::ui::binding::SelectionCommand;
use crate::ui::host::{
    EditorEventBindingDispatchError, EditorEventDispatchError, EditorEventDispatcherError,
    EditorEventExecutionError, MenuActionExecutionError,
};
use crate::ui::workbench::state::EditorStateOperationError;

struct PendingEditFixtureFactory;

impl OperationCommandFactory for PendingEditFixtureFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        Err(OperationCommandFactoryError::Factory {
            operation: invocation.operation_id.clone(),
            reason: "this fixture only verifies pending-decision publication".to_string(),
        })
    }
}

fn deferred_pending_edit_fixture() -> DeferredOperationInvocation {
    let invocation = EditorOperationInvocation::parse("editor.test.stop_retry_pending")
        .expect("pending-edit fixture operation path should be valid");
    OperationCommandFactoryRegistration::new(
        invocation.operation_id.clone(),
        "stop retry pending decision fixture",
        EditOperationTarget::EditWorkspace,
        Arc::new(PendingEditFixtureFactory),
    )
    .with_pending_edit_retention(PendingEditRetention::Lossless)
    .defer(invocation)
    .expect("fixture registration should construct a deferred pending edit")
}

#[test]
fn binding_dispatch_retains_event_execution_errors_until_the_trait_boundary() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_typed_execution_error");
    let missing_node_id = 9_999_999;
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "MissingSceneNode",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: missing_node_id,
        }),
    );

    let error = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect_err("missing scene nodes must retain their state error inside the host");
    assert!(matches!(
        error,
        EditorEventDispatcherError::Binding(
            EditorEventBindingDispatchError::EventDispatch(EditorEventDispatchError::Execution(
                EditorEventExecutionError::Selection {
                    source: EditorStateOperationError::SelectedNodeMissing { node_id },
                }
            ))
        ) if node_id == missing_node_id
    ));

    let journal = runtime.runtime.journal();
    assert_eq!(journal.records().len(), 1);
    assert_eq!(
        journal.records()[0].event,
        EditorEvent::Selection(SelectionHostEvent::SelectSceneNode {
            node_id: missing_node_id,
        })
    );
    assert_eq!(
        journal.records()[0].result.error.as_deref(),
        Some("selection event execution failed: Cannot select missing node 9999999")
    );
}

#[test]
fn exit_play_reports_stopped_runtime_when_editor_state_restore_is_blocked() {
    use crate::core::play::PlayModeKind;
    use crate::ui::workbench::startup::EditorSessionMode;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_play_stop_restore_failure");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode),
        )
        .expect("enter play mode before restore-failure coverage");
    runtime
        .runtime
        .play_sessions()
        .route_edit(
            EditOperationTarget::EditWorkspace,
            deferred_pending_edit_fixture(),
        )
        .expect("play-protected edit should remain queued until an explicit decision");
    let transition = runtime
        .runtime
        .context()
        .transactions()
        .begin(
            "test block play-state restore",
            crate::core::editing::engine::HistoryContextId::Global,
        )
        .expect("test transaction should block the editor-state exclusive transition");

    let error = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
        )
        .expect_err("a blocked state restore must remain typed after runtime stop");
    assert!(matches!(
        error,
        EditorEventDispatcherError::Event(EditorEventDispatchError::Execution(
            EditorEventExecutionError::Menu(MenuActionExecutionError::PlayStopRestoreStateFailed {
                source: EditorStateOperationError::EditCommand(_),
            })
        ))
    ));
    assert_eq!(runtime.runtime.play_sessions().mode(), PlayModeKind::Edit);
    assert_eq!(
        runtime.runtime.editor_snapshot().session_mode,
        EditorSessionMode::Playing
    );
    assert!(
        runtime
            .runtime
            .context()
            .notifications()
            .decisions()
            .expect("test notification center should be available")
            .pending_snapshot()
            .is_empty(),
        "the blocked state restore must not publish a decision before the retry can establish edit mode"
    );

    drop(transition);
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
        )
        .expect("editor state restore should be retryable after the transaction is released");
    assert_eq!(
        runtime.runtime.editor_snapshot().session_mode,
        EditorSessionMode::Project
    );
    assert_eq!(
        runtime
            .runtime
            .context()
            .notifications()
            .decisions()
            .expect("test notification center should be available")
            .pending_snapshot()
            .len(),
        1,
        "a retry after backend stop must publish the queued Apply/Discard decision"
    );
}
