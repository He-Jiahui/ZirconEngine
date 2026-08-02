use super::super::super::super::*;

pub(super) struct StartupChangeEvents {
    pub(super) asset_change_events: ChannelReceiver<AssetChange>,
    pub(super) editor_asset_change_events: EditorAssetChangeSubscription,
    pub(super) resource_change_events: zircon_runtime::core::resource::ResourceEventReceiver,
}

pub(super) fn subscribe_startup_change_events(
    asset_manager: &dyn AssetManager,
    editor_asset_manager: &dyn EditorAssetManagerContract,
    resource_manager: &dyn ResourceManager,
) -> StartupChangeEvents {
    let asset_change_events = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_subscribe_asset_changes");
        asset_manager.subscribe_asset_changes()
    };
    let editor_asset_change_events = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "new_subscribe_editor_asset_changes"
        );
        editor_asset_manager.subscribe_editor_asset_changes()
    };
    let resource_change_events = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_subscribe_resource_changes");
        resource_manager.subscribe_resource_changes()
    };

    StartupChangeEvents {
        asset_change_events,
        editor_asset_change_events,
        resource_change_events,
    }
}
