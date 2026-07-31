use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, MergeOutcome,
    SelectionSnapshot,
};
use crate::core::gateway::EditorRuntimeGatewayHandle;

pub(super) struct FixtureContext {
    pub(super) value: i32,
    pub(super) selection: u64,
    pub(super) trace: Vec<&'static str>,
    pub(super) fail_selection_restore: Option<u64>,
    pub(super) gateway: EditorRuntimeGatewayHandle,
}

impl Default for FixtureContext {
    fn default() -> Self {
        Self {
            value: 0,
            selection: 0,
            trace: Vec::new(),
            fail_selection_restore: None,
            gateway: EditorRuntimeGatewayHandle::detached(),
        }
    }
}

impl EditContext for FixtureContext {
    fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::fixture_value(self.selection, self.selection)
    }

    fn restore_selection(&mut self, snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        let selection = snapshot.fixture_value_ref()?;
        if self.fail_selection_restore == Some(selection) {
            return Err(EditCommandError::InvariantViolation {
                invariant: "fixture selection restore failure",
            });
        }
        self.selection = selection;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub(super) struct DeltaCommand {
    label: &'static str,
    key: u8,
    delta: i32,
    selection_after: Option<u64>,
    significant: bool,
    fail_apply: bool,
    mutate_before_apply_error: bool,
    fail_revert_before_mutation: bool,
    fail_revert_after_mutation: bool,
    finalized: Arc<AtomicUsize>,
}

impl DeltaCommand {
    pub(super) fn new(
        label: &'static str,
        key: u8,
        delta: i32,
        finalized: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            label,
            key,
            delta,
            selection_after: None,
            significant: true,
            fail_apply: false,
            mutate_before_apply_error: false,
            fail_revert_before_mutation: false,
            fail_revert_after_mutation: false,
            finalized,
        }
    }

    pub(super) fn selecting(mut self, selection: u64) -> Self {
        self.selection_after = Some(selection);
        self
    }

    pub(super) fn insignificant(mut self) -> Self {
        self.significant = false;
        self
    }

    pub(super) fn failing(mut self) -> Self {
        self.fail_apply = true;
        self
    }

    pub(super) fn mutating_then_failing(mut self) -> Self {
        self.fail_apply = true;
        self.mutate_before_apply_error = true;
        self
    }

    pub(super) fn revert_failing_after_mutation(mut self) -> Self {
        self.fail_revert_after_mutation = true;
        self
    }

    pub(super) fn revert_failing_before_mutation(mut self) -> Self {
        self.fail_revert_before_mutation = true;
        self
    }
}

impl EditCommand for DeltaCommand {
    fn label(&self) -> &str {
        self.label
    }

    fn is_significant(&self) -> bool {
        self.significant
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if self.fail_apply && !self.mutate_before_apply_error {
            return Err(CommandExecutionError::unchanged(
                EditCommandError::TargetMissing {
                    target: "fixture".to_string(),
                },
            ));
        }
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .ok_or(EditCommandError::ContextTypeMismatch {
                expected: "FixtureContext",
            })
            .map_err(CommandExecutionError::unchanged)?;
        fixture.value += self.delta;
        if let Some(selection) = self.selection_after {
            fixture.selection = selection;
        }
        if self.fail_apply {
            return Err(CommandExecutionError::applied(
                EditCommandError::TargetMissing {
                    target: "fixture after mutation".to_string(),
                },
            ));
        }
        fixture.trace.push(self.label);
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if self.fail_revert_before_mutation {
            return Err(CommandExecutionError::unchanged(
                EditCommandError::TargetMissing {
                    target: "fixture revert before mutation".to_string(),
                },
            ));
        }
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .ok_or(EditCommandError::ContextTypeMismatch {
                expected: "FixtureContext",
            })
            .map_err(CommandExecutionError::unchanged)?;
        fixture.value -= self.delta;
        if self.fail_revert_after_mutation {
            return Err(CommandExecutionError::applied(
                EditCommandError::TargetMissing {
                    target: "fixture revert after mutation".to_string(),
                },
            ));
        }
        fixture.trace.push("revert");
        Ok(())
    }

    fn finalize(&mut self, _context: &mut dyn EditContext) {
        self.finalized.fetch_add(1, Ordering::SeqCst);
    }

    fn try_merge(&mut self, next: &dyn EditCommand) -> MergeOutcome {
        let Some(next) = next.as_any().downcast_ref::<Self>() else {
            return MergeOutcome::Reject;
        };
        if self.key != next.key {
            return MergeOutcome::Reject;
        }
        self.delta += next.delta;
        self.selection_after = next.selection_after.or(self.selection_after);
        self.significant |= next.significant;
        MergeOutcome::Merged
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) fn finalized_counter() -> Arc<AtomicUsize> {
    Arc::new(AtomicUsize::new(0))
}
