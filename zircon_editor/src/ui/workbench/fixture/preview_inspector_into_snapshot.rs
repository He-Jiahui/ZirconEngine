use crate::ui::workbench::snapshot::InspectorSnapshot;

use super::PreviewInspector;

impl PreviewInspector {
    pub(crate) fn into_snapshot(self) -> InspectorSnapshot {
        InspectorSnapshot {
            id: self.id,
            name: self.name,
            parent: self.parent,
            translation: self.translation,
            plugin_components: Vec::new(),
        }
    }
}
