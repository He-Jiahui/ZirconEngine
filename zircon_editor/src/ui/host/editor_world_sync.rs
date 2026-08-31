use std::sync::Mutex;

use zircon_runtime_interface::world_sync::WatchRegistration;

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewayError, GatewaySessionIdentity};
use crate::core::play::WorldDomain;
use crate::core::sync::{
    QualifiedWatchToken, WorldSyncPump, WorldSyncPumpError, WorldSyncPumpReport,
    WorldSyncShutdownReceipt,
};

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn watch_world_for_view(
        &self,
        domain: WorldDomain,
        registration: WatchRegistration,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> Result<QualifiedWatchToken, WorldSyncPumpError> {
        let gateway = self.world_sync_gateway(domain)?;
        self.world_sync_pump(domain)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .watch_view_with_identity(&gateway, registration, view, mask)
    }

    pub(crate) fn unwatch_world_for_view(
        &self,
        domain: WorldDomain,
        token: &QualifiedWatchToken,
    ) -> Result<bool, WorldSyncPumpError> {
        let gateway = self.world_sync_gateway(domain)?;
        self.world_sync_pump(domain)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwatch_view(&gateway, token)
    }

    pub(crate) fn pump_world_invalidations(
        &self,
        domain: WorldDomain,
    ) -> Result<WorldSyncPumpReport, WorldSyncPumpError> {
        let gateway = self.world_sync_gateway(domain)?;
        self.world_sync_pump(domain)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pump(&gateway, self.context().bus())
    }

    pub(crate) fn acknowledge_world_replacement(
        &self,
        domain: WorldDomain,
        replacement_epoch: u64,
    ) -> bool {
        self.world_sync_pump(domain)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .acknowledge_world_replacement(replacement_epoch)
    }

    pub(crate) fn world_gateway_identity(
        &self,
        domain: WorldDomain,
    ) -> Option<GatewaySessionIdentity> {
        self.gateway_for(domain).map(|gateway| gateway.identity())
    }

    pub(in crate::ui::host) fn shutdown_play_world_sync(&self) -> WorldSyncShutdownReceipt {
        self.play_world_sync
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shutdown(&self.play_sessions.play_gateway_handle())
    }

    fn world_sync_pump(&self, domain: WorldDomain) -> &Mutex<WorldSyncPump> {
        match domain {
            WorldDomain::Edit => &self.edit_world_sync,
            WorldDomain::Play(_) => &self.play_world_sync,
        }
    }

    fn world_sync_gateway(
        &self,
        domain: WorldDomain,
    ) -> Result<EditorRuntimeGatewayHandle, WorldSyncPumpError> {
        self.gateway_for(domain)
            .ok_or(GatewayError::SessionLost)
            .map_err(Into::into)
    }
}
