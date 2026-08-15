use zircon_runtime_interface::world_sync::{WatchRegistration, WatchToken};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::sync::{WorldSyncPumpError, WorldSyncPumpReport};

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn watch_edit_world_for_view(
        &self,
        registration: WatchRegistration,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> Result<(WatchToken, u64), WorldSyncPumpError> {
        self.edit_world_sync
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .watch_view_with_gateway_generation(self.context().gateway(), registration, view, mask)
    }

    pub(crate) fn unwatch_edit_world_for_view(
        &self,
        token: WatchToken,
    ) -> Result<bool, WorldSyncPumpError> {
        self.edit_world_sync
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwatch_view(self.context().gateway(), token)
    }

    pub(crate) fn pump_edit_world_invalidations(
        &self,
    ) -> Result<WorldSyncPumpReport, WorldSyncPumpError> {
        self.edit_world_sync
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pump(self.context().gateway(), self.context().bus())
    }

    pub(crate) fn edit_world_gateway_generation(&self) -> u64 {
        self.context().gateway().generation()
    }
}
