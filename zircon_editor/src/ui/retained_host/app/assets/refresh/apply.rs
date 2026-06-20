use super::super::super::backend_refresh::AssetBackendRefreshPlan;
use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn apply_asset_refresh_plan(
        &mut self,
        plan: &AssetBackendRefreshPlan,
    ) -> Result<(), String> {
        if plan.sync_catalog {
            zircon_runtime::profile_scope!("editor", "retained_host", "asset_refresh_sync_catalog");
            self.sync_asset_catalog_snapshot();
        }
        if plan.sync_resources {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_sync_resources"
            );
            self.sync_asset_resources_snapshot();
        }
        if plan.refresh_selected_asset_details {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_selected_details"
            );
            self.refresh_selected_asset_details();
        }
        if plan.refresh_visible_asset_previews {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_visible_previews"
            );
            self.refresh_visible_asset_previews();
        }
        if plan.reload_default_scene {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_reload_default_scene"
            );
            self.reload_default_scene()?;
        }
        self.apply_asset_refresh_invalidation(plan);
        Ok(())
    }

    fn apply_asset_refresh_invalidation(&mut self, plan: &AssetBackendRefreshPlan) {
        let mut invalidation = HostInvalidationMask::NONE;
        if plan.mark_render_dirty {
            invalidation.insert(HostInvalidationMask::RENDER);
        }
        if plan.mark_presentation_dirty {
            invalidation.insert(HostInvalidationMask::PRESENTATION_DATA);
        }
        if !invalidation.is_empty() {
            zircon_runtime::profile_scope!("editor", "retained_host", "asset_refresh_invalidate");
            self.invalidate_host(invalidation);
        }
        if plan.mark_paint_only_dirty {
            zircon_runtime::profile_scope!("editor", "retained_host", "asset_refresh_paint_only");
            self.record_paint_only_invalidation(HostInvalidationMask::PAINT_ONLY);
            self.request_asset_preview_paint_only_redraw();
        }
    }
}
