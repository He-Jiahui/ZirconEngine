use crate::snapshot::EditorChromeSnapshot;

use super::PreviewFixture;

impl PreviewFixture {
    pub fn build_chrome(&self) -> EditorChromeSnapshot {
        EditorChromeSnapshot::build(
            self.editor.clone().into_snapshot(),
            &self.layout,
            self.instances.clone(),
            self.descriptors.clone(),
        )
    }
}
