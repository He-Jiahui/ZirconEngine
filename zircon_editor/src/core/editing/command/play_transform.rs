use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::{ZrRuntimeEditorTransformPhaseV1, ZrRuntimeEditorTransformWriteV1};

use super::super::context::CoreEditContext;
use super::super::engine::{
    CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable,
};

#[derive(Clone, Debug)]
pub(super) struct PlayTransformCommand {
    pub(super) node_id: NodeId,
    interaction_id: u64,
    pub(super) world_replacement_epoch: u64,
    before: Transform,
    pub(super) after: Transform,
    pub(super) already_applied: bool,
}

impl PlayTransformCommand {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        node_id: NodeId,
        interaction_id: u64,
        world_replacement_epoch: u64,
        before: Transform,
        after: Transform,
        already_applied: bool,
    ) -> Option<Self> {
        (before != after).then_some(Self {
            node_id,
            interaction_id,
            world_replacement_epoch,
            before,
            after,
            already_applied,
        })
    }

    pub(super) fn apply(
        &mut self,
        context: &mut CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        if self.already_applied {
            self.already_applied = false;
            return Ok(());
        }
        self.dispatch(context, self.before, self.after)
    }

    pub(super) fn revert(
        &mut self,
        context: &mut CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        self.dispatch(context, self.after, self.before)
    }

    fn dispatch(
        &self,
        context: &CoreEditContext,
        expected: Transform,
        target: Transform,
    ) -> Result<(), CommandExecutionError> {
        let request = ZrRuntimeEditorTransformWriteV1::new(
            self.node_id,
            self.interaction_id,
            1,
            self.world_replacement_epoch,
            ZrRuntimeEditorTransformPhaseV1::Apply,
            expected,
            target,
        );
        context
            .dispatch_runtime_transform(&request)
            .map_err(CommandExecutionError::unchanged)
    }

    pub(super) fn journal_payload(
        &self,
    ) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        Err(CommandJournalUnavailable::new("Transform Play scene node"))
    }
}
