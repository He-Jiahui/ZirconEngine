use super::{AssetRefreshEvents, RetainedEditorHost};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::assets::refresh) fn drain_asset_refresh_events(
        &mut self,
    ) -> AssetRefreshEvents {
        let mut asset_changes = Vec::new();
        while let Ok(change) = self.asset_change_events.try_recv() {
            asset_changes.push(change);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.asset_change_count",
            asset_changes.len()
        );

        let mut editor_asset_changes = Vec::new();
        while let Ok(change) = self.editor_asset_change_events.try_recv() {
            editor_asset_changes.push(change);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.editor_change_count",
            editor_asset_changes.len()
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

        AssetRefreshEvents {
            asset_changes,
            editor_asset_changes,
            resource_changes,
        }
    }
}
