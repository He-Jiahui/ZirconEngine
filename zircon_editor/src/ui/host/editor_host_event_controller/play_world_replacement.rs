use thiserror::Error;

use crate::core::editing::engine::EditCommandError;
use crate::core::gateway::GatewaySessionIdentity;
use crate::core::play::{PlayInstanceId, WorldDomain};

use super::EditorHostEventController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlayWorldReplacementRetirementReport {
    replacement_epoch: u64,
    history_discarded: bool,
    selection_cleared: bool,
    hierarchy_cleared: bool,
    inspector_cleared: bool,
}

impl PlayWorldReplacementRetirementReport {
    pub(crate) const fn replacement_epoch(self) -> u64 {
        self.replacement_epoch
    }

    pub(crate) const fn history_discarded(self) -> bool {
        self.history_discarded
    }

    pub(crate) const fn selection_cleared(self) -> bool {
        self.selection_cleared
    }

    pub(crate) const fn hierarchy_cleared(self) -> bool {
        self.hierarchy_cleared
    }

    pub(crate) const fn inspector_cleared(self) -> bool {
        self.inspector_cleared
    }
}

#[derive(Debug, Error)]
pub(crate) enum PlayWorldReplacementRetirementError {
    #[error("runtime published the reserved zero Play world replacement epoch")]
    ZeroReplacementEpoch,
    #[error("Play world replacement belongs to a stale gateway")]
    StaleGateway,
    #[error(transparent)]
    History(#[from] EditCommandError),
}

impl EditorHostEventController {
    pub(crate) fn retire_replaced_play_world(
        &self,
        instance: PlayInstanceId,
        identity: &GatewaySessionIdentity,
        replacement_epoch: u64,
    ) -> Result<PlayWorldReplacementRetirementReport, PlayWorldReplacementRetirementError> {
        if replacement_epoch == 0 {
            return Err(PlayWorldReplacementRetirementError::ZeroReplacementEpoch);
        }
        let domain = WorldDomain::Play(instance);
        if self.play_sessions.attached_world_domain() != Some(domain)
            || self.world_gateway_identity(domain).as_ref() != Some(identity)
        {
            return Err(PlayWorldReplacementRetirementError::StaleGateway);
        }

        let history_discarded = self.context.transactions().discard_play_history(instance)?;
        self.retire_play_gizmo_local_state();
        let selection_cleared = self
            .shell
            .lock()
            .state
            .viewport_controller
            .selection_mut()
            .clear(domain);
        let hierarchy_cleared = self
            .play_hierarchy_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        let inspector_cleared = self
            .play_inspector_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        zircon_runtime::profile_counter!("editor", "play.world_replacement.retired_count", 1);

        Ok(PlayWorldReplacementRetirementReport {
            replacement_epoch,
            history_discarded,
            selection_cleared,
            hierarchy_cleared,
            inspector_cleared,
        })
    }
}

#[cfg(test)]
#[path = "tests/play_world_replacement.rs"]
mod tests;
