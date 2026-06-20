use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn drain_initial_asset_refresh_events(&mut self) {
        // Bootstrap loads snapshots and the default scene; queued events from
        // that work should not replay as a second full presentation rebuild.
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
}
