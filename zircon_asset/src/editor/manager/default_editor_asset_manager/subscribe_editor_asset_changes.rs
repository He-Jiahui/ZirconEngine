use crossbeam_channel::unbounded;
use zircon_core::ChannelReceiver;

use super::super::super::EditorAssetChangeRecord;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn subscribe_editor_asset_changes_impl(
        &self,
    ) -> ChannelReceiver<EditorAssetChangeRecord> {
        let (sender, receiver) = unbounded();
        self.change_subscribers
            .lock()
            .expect("editor asset subscriber lock poisoned")
            .push(sender);
        receiver
    }
}
