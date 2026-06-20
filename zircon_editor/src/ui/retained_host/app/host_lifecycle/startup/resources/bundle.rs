use std::sync::Arc;

use super::super::super::super::*;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) struct StartupManagers {
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) asset_manager:
        Arc<dyn AssetManager>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) editor_asset_manager:
        Arc<dyn EditorAssetManagerContract>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) resource_manager:
        Arc<dyn ResourceManager>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) editor_manager:
        Arc<EditorManager>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) asset_change_events:
        ChannelReceiver<AssetChange>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) editor_asset_change_events:
        ChannelReceiver<EditorAssetChange>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) resource_change_events:
        ChannelReceiver<ResourceEvent>,
}
