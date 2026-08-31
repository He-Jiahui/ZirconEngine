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
