use super::super::backend_refresh::{plan_asset_backend_refresh, AssetBackendRefreshPlan};
use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn refresh_project_assets(
        &mut self,
    ) -> Result<(), String> {
        zircon_runtime::profile_scope!("editor", "retained_host", "refresh_project_assets");
        let mut changes = Vec::new();
        while let Ok(change) = self.asset_change_events.try_recv() {
            changes.push(change);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.asset_change_count",
            changes.len()
        );
        if !changes.is_empty() {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_runtime_project"
            );
            self.editor_asset_manager
                .refresh_from_runtime_project()
                .map_err(|error| error.to_string())?;
        }
        let mut editor_changes = Vec::new();
        while let Ok(change) = self.editor_asset_change_events.try_recv() {
            editor_changes.push(change);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.editor_change_count",
            editor_changes.len()
        );
        let mut resource_changes = Vec::new();
        while let Ok(change) = self.resource_change_events.try_recv() {
            resource_changes.push(change);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.resource_change_count",
            resource_changes.len()
        );
        if changes.is_empty() && editor_changes.is_empty() && resource_changes.is_empty() {
            return Ok(());
        }

        let selected_asset_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        let default_scene_uri = self
            .asset_manager
            .current_project()
            .map(|project| project.default_scene_uri);
        let plan = plan_asset_backend_refresh(
            selected_asset_uuid.as_deref(),
            default_scene_uri.as_deref(),
            &changes,
            &editor_changes,
            &resource_changes,
        );
        record_asset_refresh_plan_counters(&plan);

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

        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn drain_initial_asset_refresh_events(&mut self) {
        // The startup state has just pulled catalog/resource snapshots and
        // loaded the default scene. Events already queued by that bootstrap
        // work should not replay as a second full presentation rebuild.
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "startup_drain_initial_asset_refresh_events"
        );
        let mut asset_count = 0usize;
        while self.asset_change_events.try_recv().is_ok() {
            asset_count += 1;
        }
        let mut editor_count = 0usize;
        while self.editor_asset_change_events.try_recv().is_ok() {
            editor_count += 1;
        }
        let mut resource_count = 0usize;
        while self.resource_change_events.try_recv().is_ok() {
            resource_count += 1;
        }
        #[cfg(not(feature = "profiling"))]
        let _ = (asset_count, editor_count, resource_count);
        zircon_runtime::profile_counter!(
            "editor",
            "ui.startup.drained_asset_change_count",
            asset_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.startup.drained_editor_asset_change_count",
            editor_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.startup.drained_resource_change_count",
            resource_count
        );
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_catalog(&mut self) {
        self.sync_asset_catalog_snapshot();
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    fn sync_asset_catalog_snapshot(&mut self) {
        self.runtime
            .sync_asset_catalog(self.editor_asset_manager.catalog_snapshot());
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_resources(&mut self) {
        self.sync_asset_resources_snapshot();
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    fn sync_asset_resources_snapshot(&mut self) {
        self.runtime
            .sync_asset_resources(self.resource_manager.list_resources());
    }

    pub(in crate::ui::retained_host::app) fn refresh_selected_asset_details(&mut self) {
        let selected_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        self.runtime.sync_asset_details(
            selected_uuid
                .as_deref()
                .and_then(|uuid| self.editor_asset_manager.asset_details(uuid)),
        );
    }

    pub(in crate::ui::retained_host::app) fn refresh_visible_asset_previews(&mut self) {
        if self.asset_manager.current_project().is_none() {
            return;
        }

        let chrome = self.build_chrome();
        let mut visible = BTreeSet::new();

        if asset_surface_visible(&chrome, ViewContentKind::Assets) {
            visible.extend(
                chrome
                    .asset_activity
                    .visible_assets
                    .iter()
                    .map(|asset| asset.uuid.clone()),
            );
            if let Some(uuid) = chrome.asset_activity.selection.uuid.clone() {
                visible.insert(uuid);
            }
        }

        if asset_surface_visible(&chrome, ViewContentKind::AssetBrowser) {
            visible.extend(
                chrome
                    .asset_browser
                    .visible_assets
                    .iter()
                    .map(|asset| asset.uuid.clone()),
            );
            if let Some(uuid) = chrome.asset_browser.selection.uuid.clone() {
                visible.insert(uuid);
            }
        }

        for uuid in visible {
            let _ = self
                .editor_asset_manager
                .request_preview_refresh(&uuid, true);
        }
    }

    fn request_asset_preview_paint_only_redraw(&self) {
        let frame = self.ui.get_host_window_bootstrap().shell_frame;
        self.ui.request_redraw_region(frame);
    }
}

fn record_asset_refresh_plan_counters(plan: &AssetBackendRefreshPlan) {
    #[cfg(not(feature = "profiling"))]
    let _ = plan;
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_sync_catalog",
        plan.sync_catalog as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_sync_resources",
        plan.sync_resources as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_refresh_selected_asset_details",
        plan.refresh_selected_asset_details as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_refresh_visible_asset_previews",
        plan.refresh_visible_asset_previews as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_reload_default_scene",
        plan.reload_default_scene as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_mark_render_dirty",
        plan.mark_render_dirty as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_mark_presentation_dirty",
        plan.mark_presentation_dirty as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_mark_paint_only_dirty",
        plan.mark_paint_only_dirty as u8
    );
}
