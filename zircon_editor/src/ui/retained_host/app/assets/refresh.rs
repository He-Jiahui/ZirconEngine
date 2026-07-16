use super::super::backend_refresh::plan_asset_backend_refresh;
use super::super::*;
use counters::record_asset_refresh_plan_counters;

mod apply;
mod counters;
mod events;
mod snapshots;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn refresh_project_assets(
        &mut self,
    ) -> Result<(), String> {
        zircon_runtime::profile_scope!("editor", "retained_host", "refresh_project_assets");
        let events = self.drain_asset_refresh_events();
        if !events.asset_changes.is_empty() {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_runtime_project"
            );
            self.editor_asset_manager_at_use_point()
                .map_err(|error| error.to_string())?
                .refresh_from_runtime_project()
                .map_err(|error| error.to_string())?;
        }
        if events.is_empty() {
            return Ok(());
        }

        let selected_asset_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        let default_scene_uri = self
            .asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .current_project()
            .map(|project| project.default_scene_uri);
        let plan = plan_asset_backend_refresh(
            selected_asset_uuid.as_deref(),
            default_scene_uri.as_deref(),
            &events.asset_changes,
            &events.editor_asset_changes,
            &events.resource_changes,
        );
        record_asset_refresh_plan_counters(&plan);
        self.apply_asset_refresh_plan(&plan)
    }
}
