use crate::snapshot::SceneEntry;

use super::PreviewSceneEntry;

impl PreviewSceneEntry {
    pub(crate) fn into_snapshot(self) -> SceneEntry {
        SceneEntry {
            id: self.id,
            name: self.name,
            depth: self.depth,
            selected: self.selected,
        }
    }
}
