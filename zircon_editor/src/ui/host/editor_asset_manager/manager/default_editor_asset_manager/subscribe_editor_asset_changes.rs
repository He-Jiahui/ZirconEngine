use super::super::super::EditorAssetChangeSubscription;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn subscribe_editor_asset_changes_impl(&self) -> EditorAssetChangeSubscription {
        self.change_stream.subscribe()
    }
}
