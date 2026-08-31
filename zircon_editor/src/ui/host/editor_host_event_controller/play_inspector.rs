use std::time::Instant;

use zircon_runtime_interface::world_sync::WorldQuery;

use crate::core::play::{PlayInstanceId, WorldDomain};
use crate::ui::workbench::snapshot::InspectorSnapshot;

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn refresh_play_inspector_if_due(
        &self,
        instance: PlayInstanceId,
        now: Instant,
    ) -> Result<bool, String> {
        let domain = WorldDomain::Play(instance);
        let gateway = self
            .gateway_for(domain)
            .ok_or_else(|| format!("play Inspector gateway is unavailable for {instance:?}"))?;
        let entity = {
            let shell = self.shell().lock();
            shell.state.viewport_controller.selection().active_primary()
        };
        let Some(entity) = entity else {
            return Ok(self
                .play_inspector_projection
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear());
        };
        let identity = gateway.identity();
        let Some(generation_hint) = self
            .play_inspector_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .begin_query(&identity, entity, now)
        else {
            return Ok(false);
        };
        let result = gateway
            .query_world_at_identity(
                &identity,
                WorldQuery::inspection_fields(entity, generation_hint),
            )
            .map_err(|error| format!("play Inspector query failed: {error}"))?;
        let hierarchy_row = self
            .play_hierarchy_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .row(entity);
        self.play_inspector_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .apply(identity, entity, result, hierarchy_row.as_ref())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn clear_play_inspector(&self) -> bool {
        self.play_inspector_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear()
    }

    pub(crate) fn play_inspector_snapshot(&self) -> Option<InspectorSnapshot> {
        let WorldDomain::Play(instance) = self.active_hierarchy_world_domain() else {
            return None;
        };
        let gateway = self.gateway_for(WorldDomain::Play(instance))?;
        let entity = {
            let shell = self.shell().lock();
            shell
                .state
                .viewport_controller
                .selection()
                .active_primary()?
        };
        self.play_inspector_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot_for(&gateway.identity(), entity)
    }
}
