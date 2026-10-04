use zircon_runtime::core::framework::scene::EntityId;

use crate::core::play::{PlayInstanceId, PlayKind, PlayMode, WorldDomain};
use crate::scene::selection::SelectionMutation;

use super::EditorHostEventController;

impl EditorHostEventController {
    /// Applies a renderer-owned SIE pick only to the matching active Play selection domain.
    ///
    /// `None` rejects a late or cross-session completion. `Some(false)` is a valid completion that
    /// happened to preserve the current selection.
    pub(crate) fn apply_play_viewport_pick_selection(
        &self,
        instance: PlayInstanceId,
        entity: Option<EntityId>,
        mutation: SelectionMutation,
    ) -> Option<bool> {
        if !matches!(
            self.play_sessions().mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) || self.play_sessions().attached_world_domain() != Some(WorldDomain::Play(instance))
        {
            return None;
        }

        let changed = {
            let mut shell = self.shell().lock();
            if !shell.state.is_playing()
                || shell.state.viewport_controller.selection().active_domain()
                    != WorldDomain::Play(instance)
            {
                return None;
            }
            shell
                .state
                .viewport_controller
                .selection_mut()
                .apply_active(entity, mutation)
        };
        if changed {
            self.play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .invalidate_projection();
        }
        Some(changed)
    }
}

#[cfg(test)]
#[path = "tests/play_viewport_pick.rs"]
mod tests;
