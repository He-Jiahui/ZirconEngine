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
    use std::sync::{Arc, Mutex};
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
        fn selection_retention_bound_v1(
            &self,
            model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
            limit: u64,
        ) -> Result<u64, EditCommandError> {
            SelectionSnapshot::scalar_retention_bound_v1(model, limit)
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
    struct Add(Arc<Mutex<i64>>, i64);
    impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for Add {
        fn retained_bound_v1(
            &self,
            m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
        ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
            m.owned(&self.0)?;
            Ok(())
        }
    }
    impl EditCommand for Add {
        fn retention_plan_v1(
            &self,
            context: &dyn crate::core::editing::engine::EditContext,
            phase: crate::core::editing::engine::RetentionPhaseV1,
            model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
            limit: u64,
        ) -> Result<
            crate::core::editing::engine::CommandRetentionPlanV1,
            crate::core::editing::engine::EditCommandError,
        > {
            crate::core::editing::engine::CommandRetentionPlanV1::for_owner(
                self, context, phase, model, limit,
            )
        }
        fn prepare_retention_v1(
            &mut self,
            _: &mut dyn crate::core::editing::engine::EditContext,
            phase: crate::core::editing::engine::RetentionPhaseV1,
            permit: &mut crate::core::editing::engine::RetentionMaterializationV1,
        ) -> Result<(), crate::core::editing::engine::CommandExecutionError> {
            permit
                .consume(self, phase)
                .map_err(crate::core::editing::engine::CommandExecutionError::unchanged)
        }

        fn label(&self) -> &str {
            "close"
        }
        fn apply(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            *self.0.lock().unwrap() += self.1;
            Ok(())
        }
        fn revert(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            *self.0.lock().unwrap() -= self.1;
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
    let command = |value: &Arc<Mutex<i64>>| Add(Arc::clone(value), 1);
    let engine = EditorTransactionEngine::new(Context);
    let history = HistoryContextId::Global;
    let value = Arc::new(Mutex::new(0));
    let mut scope = engine.begin("close", history).unwrap();
    scope.set_merge_mode(MergeMode::Ends);
    scope.push(command(&value)).unwrap();
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
    scope.push(command(&value)).unwrap();
    let current = engine
        .dirty_history_decision_token(history)
        .unwrap()
        .unwrap();
    assert!(!prompt.permits_discard(&[], Some(&current)));
    prompt.finish_save(Vec::new(), Some(current.clone()));
    assert!(prompt.permits_discard(&[], Some(&current)));
    scope.cancel().unwrap();
    assert_eq!(*value.lock().unwrap(), 0);
    assert!(prompt.permits_discard(&[], None));
}
