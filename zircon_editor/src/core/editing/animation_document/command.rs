use std::any::Any;

use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext,
};
use crate::core::editor_message::DocumentId;

use super::{AnimationAuthoringAsset, AnimationDocumentRevision};

/// A CAS-guarded source swap. Apply and revert use the same operation, which keeps undo/redo
/// source exact while monotonically advancing the document revision.
pub(crate) struct AnimationEditCommand {
    label: &'static str,
    document: DocumentId,
    expected_revision: AnimationDocumentRevision,
    replacement: AnimationAuthoringAsset,
}

impl AnimationEditCommand {
    pub(crate) fn new(
        label: &'static str,
        document: DocumentId,
        expected_revision: AnimationDocumentRevision,
        replacement: AnimationAuthoringAsset,
    ) -> Self {
        Self {
            label,
            document,
            expected_revision,
            replacement,
        }
    }

    fn swap(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let context = context
            .as_any_mut()
            .downcast_mut::<CoreEditContext>()
            .ok_or_else(|| {
                CommandExecutionError::unchanged(EditCommandError::ContextTypeMismatch {
                    expected: std::any::type_name::<CoreEditContext>(),
                })
            })?;
        let mut document = context
            .animation_documents()
            .document_mut(self.document)
            .map_err(|error| {
                CommandExecutionError::unchanged(EditCommandError::TargetMissing {
                    target: error.to_string(),
                })
            })?;
        self.expected_revision = document
            .swap_asset_if_revision(self.expected_revision, &mut self.replacement)
            .map_err(|error| {
                CommandExecutionError::unchanged(EditCommandError::InvariantViolation {
                    invariant: match error {
                        super::AnimationAuthoringDocumentError::StaleRevision { .. } => {
                            "animation document revision changed before transaction apply"
                        }
                        super::AnimationAuthoringDocumentError::WrongKind { .. } => {
                            "animation edit command attempted to replace a different document kind"
                        }
                        super::AnimationAuthoringDocumentError::RevisionExhausted => {
                            "animation document revision space is exhausted"
                        }
                        _ => "animation authoring document source is unavailable",
                    },
                })
            })?;
        Ok(())
    }
}

impl EditCommand for AnimationEditCommand {
    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<crate::core::editing::engine::CommandRetentionPlanV1, EditCommandError> {
        use crate::core::editing::engine::*;
        use zircon_runtime_interface::retained_memory::*;
        let mut plan = CommandRetentionPlanV1::for_owner(self, context, phase, model, limit)?;
        let context = context.as_any().downcast_ref::<CoreEditContext>().ok_or(
            EditCommandError::ContextTypeMismatch {
                expected: "CoreEditContext",
            },
        )?;
        let handle = context
            .animation_documents()
            .handle(self.document)
            .map_err(|error| EditCommandError::TargetMissing {
                target: error.to_string(),
            })?;
        let document = handle.read();
        if document.revision() != self.expected_revision {
            return Err(RetentionContractError::Stale.into());
        }
        let mut meter = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        meter.owned(document.asset())?;
        plan.add_retained(meter.bytes().get())?;
        // The exact alternate source is rechecked while holding the authority's
        // read guard; the CAS swap remains the sole publication point.
        Ok(plan)
    }
    fn prepare_retention_v1(
        &mut self,
        context: &mut dyn EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        permit: &mut crate::core::editing::engine::RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError> {
        permit
            .consume(self, phase)
            .map_err(CommandExecutionError::unchanged)?;
        self.retention_plan_v1(context, phase, permit.model(), permit.plan().retained_bytes)
            .map_err(CommandExecutionError::unchanged)?;
        Ok(())
    }

    fn label(&self) -> &str {
        self.label
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.swap(context)
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.swap(context)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for AnimationAuthoringAsset {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        match self {
            Self::Sequence(v) => v.retained_bound_v1(m),
            Self::Graph(v) => v.retained_bound_v1(m),
            Self::StateMachine(v) => v.retained_bound_v1(m),
        }
    }
}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for AnimationEditCommand {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        zircon_runtime_interface::retained_memory::RetainedOwnerV1::retained_bound_v1(
            &self.replacement,
            m,
        )
    }
}
