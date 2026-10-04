use crate::core::editor_message::DocumentId;
use crate::ui::workbench::view::ViewInstanceId;

use super::{ClosePromptTarget, DirtyCloseView, DocumentCloseRevision, PendingClosePrompt};

fn dirty_view(document: u64, generation: u64, instance: &str) -> DirtyCloseView {
    DirtyCloseView {
        document_id: DocumentId::new(document),
        dirty_generation: generation,
        close_revision: DocumentCloseRevision {
            external_generation: generation,
            ..Default::default()
        },
        instance_id: ViewInstanceId::new(instance),
        title: instance.to_string(),
    }
}

#[test]
fn discard_requires_every_current_dirty_document_to_match_the_captured_plan() {
    let planned = dirty_view(7, 3, "editor.asset#7");
    let prompt = PendingClosePrompt::new(
        ClosePromptTarget::Project,
        Vec::new(),
        vec![planned.clone()],
    );

    assert!(prompt.permits_discard(&[planned], None));
    assert!(!prompt.permits_discard(&[dirty_view(7, 4, "editor.asset#7")], None));
    assert!(!prompt.permits_discard(&[dirty_view(8, 1, "editor.asset#8")], None));
}

#[test]
fn discard_requires_a_dirty_scene_generation_to_match_the_captured_plan() {
    use crate::core::editing::engine::{
        CommandExecutionError, EditCommand, EditCommandError, EditContext, EditWorldRoute,
        EditorTransactionEngine, HistoryContextId, MergeMode, MergeOutcome, SelectionSnapshot,
    };
    use crate::core::play::WorldDomain;
    use std::any::Any;
    use std::sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    };
    struct Context;
    impl EditContext for Context {
        fn capture_world_route(
            &self,
            domain: WorldDomain,
        ) -> Result<EditWorldRoute, EditCommandError> {
            Ok(EditWorldRoute::logical(domain))
        }
        fn activate_world_route(&mut self, _: &EditWorldRoute) -> Result<(), EditCommandError> {
            Ok(())
        }
        fn retire_world_route(&mut self, _: WorldDomain) -> Result<(), EditCommandError> {
            Ok(())
        }
        fn selection_snapshot(&self) -> SelectionSnapshot {
            SelectionSnapshot::default()
        }
        fn restore_selection(&mut self, _: &SelectionSnapshot) -> Result<(), EditCommandError> {
            Ok(())
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    struct Add(Arc<AtomicI64>, i64);
    impl EditCommand for Add {
        fn label(&self) -> &str {
            "close"
        }
        fn apply(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            self.0.fetch_add(self.1, Ordering::SeqCst);
            Ok(())
        }
        fn revert(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            self.0.fetch_sub(self.1, Ordering::SeqCst);
            Ok(())
        }
        fn try_merge(&mut self, next: &dyn EditCommand) -> MergeOutcome {
            let Some(next) = next.as_any().downcast_ref::<Self>() else {
                return MergeOutcome::Reject;
            };
            self.1 += next.1;
            MergeOutcome::Merged
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    let command = |value: &Arc<AtomicI64>| Add(Arc::clone(value), 1);
    let engine = EditorTransactionEngine::new(Context);
    let history = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = engine
        .execute_operation(
            "close",
            history,
            Some("close"),
            MergeMode::Ends,
            Box::new(command(&value)),
        )
        .unwrap()
        .transaction_id;
    let first = engine
        .dirty_history_decision_token(history)
        .unwrap()
        .unwrap();
    let mut prompt = PendingClosePrompt::new(ClosePromptTarget::Project, Vec::new(), Vec::new())
        .with_dirty_project_scene(first.clone());
    assert!(prompt.permits_discard(&[], Some(&first)));
    prompt.begin_save();
    prompt.finish_save_failed();
    assert!(!prompt.save_in_flight());
    assert!(prompt.has_dirty_project_scene());
    assert!(prompt.permits_discard(&[], Some(&first)));
    engine
        .execute_operation(
            "close",
            history,
            Some("close"),
            MergeMode::Ends,
            Box::new(command(&value)),
        )
        .unwrap();
    let current = engine
        .dirty_history_decision_token(history)
        .unwrap()
        .unwrap();
    assert!(!prompt.permits_discard(&[], Some(&current)));
    prompt.finish_save(Vec::new(), Some(current.clone()));
    assert!(prompt.permits_discard(&[], Some(&current)));
    engine.cancel(id).unwrap();
    assert!(prompt.permits_discard(&[], None));
}
