use super::super::super::EditorAssetChangeRecord;
use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(crate) fn broadcast(&self, change: EditorAssetChangeRecord) {
        self.change_stream.publish(change);
    }
}
