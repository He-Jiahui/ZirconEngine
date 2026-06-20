use super::super::super::super::RetainedEditorHost;

pub(super) fn finalize_startup_host(host: &mut RetainedEditorHost) {
    host.sync_asset_workspace();
    host.drain_initial_asset_refresh_events();
    host.publish_refresh_invalidation_diagnostics();
}
