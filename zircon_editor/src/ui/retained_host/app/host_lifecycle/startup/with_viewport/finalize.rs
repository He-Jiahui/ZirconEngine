use super::super::super::super::RetainedEditorHost;

// 首帧前先统一资产工作区快照并清空启动期事件积压，然后发布失效诊断。
pub(super) fn finalize_startup_host(host: &mut RetainedEditorHost) {
    host.sync_asset_workspace();
    host.drain_initial_asset_refresh_events();
    host.publish_refresh_invalidation_diagnostics();
}
