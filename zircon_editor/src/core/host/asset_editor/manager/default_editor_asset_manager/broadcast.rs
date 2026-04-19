use super::super::super::EditorAssetChangeRecord;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn broadcast(&self, change: EditorAssetChangeRecord) {
        let mut subscribers = self
            .change_subscribers
            .lock()
            .expect("editor asset subscriber lock poisoned");
        subscribers.retain(|sender| sender.send(change.clone()).is_ok());
    }
}
