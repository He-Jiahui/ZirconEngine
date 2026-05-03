use super::super::super::EditorAssetChangeRecord;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn broadcast(&self, change: EditorAssetChangeRecord) {
        let mut subscribers = self.lock_change_subscribers();
        subscribers.retain(|sender| sender.send(change.clone()).is_ok());
    }
}
