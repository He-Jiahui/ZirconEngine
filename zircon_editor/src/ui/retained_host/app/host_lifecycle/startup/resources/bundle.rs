use std::sync::Arc;

use super::super::super::super::*;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) struct StartupManagers {
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) asset_runtime_access:
        RetainedHostAssetRuntimeAccess,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) editor_manager:
        Arc<EditorManager>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) asset_change_events:
        ChannelReceiver<AssetChange>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) editor_asset_change_events:
        EditorAssetChangeSubscription,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) resource_change_events:
        zircon_runtime::core::resource::ResourceEventReceiver,
}
