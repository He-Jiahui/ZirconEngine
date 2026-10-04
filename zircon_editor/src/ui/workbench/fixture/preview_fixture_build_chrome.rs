use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::PreviewFixture;

impl PreviewFixture {
    /// 为可重复的预览断言构造chrome快照；默认不声明聚焦视图。
    pub fn build_chrome(&self) -> EditorChromeSnapshot {
        EditorChromeSnapshot::build(
            self.editor.clone().into_snapshot(),
            &self.layout,
            self.instances.clone(),
            self.descriptors.clone(),
            None,
        )
    }
}
