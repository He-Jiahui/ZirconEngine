use zircon_runtime_interface::world_sync::WorldQuery;

use crate::core::play::{PlayInstanceId, WorldDomain};
use crate::ui::workbench::snapshot::SceneInspectionHierarchyFragment;

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn sync_active_selection_world_domain(&self) -> bool {
        let domain = self.active_hierarchy_world_domain();
        self.shell()
            .lock()
            .state
            .sync_selection_world_domain(domain)
    }

    pub(crate) fn active_hierarchy_world_domain(&self) -> WorldDomain {
        if self.shell().lock().state.is_playing() {
            if let Some(domain @ WorldDomain::Play(_)) = self.play_sessions.attached_world_domain()
            {
                return domain;
            }
        }
        WorldDomain::Edit
    }

    pub(crate) fn query_play_hierarchy_fragment(
        &self,
        instance: PlayInstanceId,
        force_reflow: bool,
    ) -> Result<Option<SceneInspectionHierarchyFragment>, String> {
        let domain = WorldDomain::Play(instance);
        let gateway = self
            .gateway_for(domain)
            .ok_or_else(|| format!("play hierarchy gateway is unavailable for {instance:?}"))?;
        let identity = gateway.identity();
        let generation_hint = (!force_reflow)
            .then(|| {
                self.play_hierarchy_projection
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .generation_hint(&identity)
            })
            .flatten();
        let result = gateway
            .query_world_at_identity(&identity, WorldQuery::hierarchy(generation_hint))
            .map_err(|error| format!("play hierarchy query failed: {error}"))?;
        let (focused_entity, selection_revision, selected_entities) = {
            let shell = self.shell().lock();
            let selection = shell.state.viewport_controller.selection();
            (
                selection.active_primary(),
                selection.revision(),
                selection.active_items().iter().copied().collect::<Vec<_>>(),
            )
        };
        self.play_hierarchy_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .apply(
                identity,
                result,
                focused_entity,
                selection_revision,
                selected_entities,
                force_reflow,
            )
            .map_err(|error| error.to_string())
    }
}
