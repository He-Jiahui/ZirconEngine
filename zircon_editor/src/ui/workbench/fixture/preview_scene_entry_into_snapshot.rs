use crate::ui::workbench::snapshot::SceneEntry;

use super::PreviewSceneEntry;

impl PreviewSceneEntry {
    /// 只转换层级行；选择身份由上游聚合后交给SceneEntries。
    pub(crate) fn into_snapshot(self) -> SceneEntry {
        SceneEntry {
            id: self.id,
            name: self.name,
            depth: self.depth,
        }
    }
}
