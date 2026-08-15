use super::super::super::EditorAssetChangeSubscription;
use super::DefaultEditorAssetManager;
use zircon_runtime::core::framework::channel::ChannelWakeCallback;

impl DefaultEditorAssetManager {
    pub(crate) fn subscribe_editor_asset_changes_impl(&self) -> EditorAssetChangeSubscription {
        self.change_stream.subscribe()
    }

    pub(crate) fn subscribe_editor_asset_changes_with_wake_impl(
        &self,
        wake: ChannelWakeCallback,
    ) -> EditorAssetChangeSubscription {
        self.change_stream.subscribe_with_wake(wake)
    }
}
