use super::super::super::*;
use std::collections::BTreeSet;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_asset_catalog(&mut self) {
        self.sync_asset_catalog_snapshot();
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(super) fn sync_asset_catalog_snapshot(&mut self) {
        if let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() {
            self.runtime
                .sync_asset_catalog(editor_asset_manager.catalog_snapshot());
        }
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_resources(&mut self) {
        if self.sync_asset_resources_snapshot() {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        }
    }

    pub(super) fn sync_asset_resources_snapshot(&mut self) -> bool {
        if let Ok(resource_manager) = self.resolve_resource_manager() {
            return self
                .runtime
                .sync_asset_resources(resource_manager.resource_management_generation());
        }
        false
    }

    pub(in crate::ui::retained_host::app) fn refresh_selected_asset_details(&mut self) {
        let selected_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        let details = self
            .editor_asset_manager_at_use_point()
            .ok()
            .and_then(|manager| {
                selected_uuid
                    .as_deref()
                    .and_then(|uuid| manager.asset_details(uuid))
            });
        self.runtime.sync_asset_details(details);
    }

    pub(in crate::ui::retained_host::app) fn refresh_visible_asset_previews(&mut self) {
        let Ok(asset_manager) = self.asset_manager_at_use_point() else {
            return;
        };
        if asset_manager.current_project().is_none() {
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

        let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() else {
            return;
        };
        for uuid in visible {
            let _ = editor_asset_manager.request_preview_refresh(&uuid, true);
        }
    }

    pub(super) fn request_asset_preview_paint_only_redraw(&self) {
        let frame = self.ui.get_host_window_bootstrap().shell_frame;
        self.ui.request_redraw_region(frame);
    }
}
