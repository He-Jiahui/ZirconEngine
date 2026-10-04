use crate::ui::workbench::snapshot::InspectorSnapshot;

use super::PreviewInspector;

impl PreviewInspector {
    /// 为预览填充Inspector快照契约；额外组件领域留作空样本。
    pub(crate) fn into_snapshot(self) -> InspectorSnapshot {
        InspectorSnapshot {
            rotation_degrees: None,
            id: self.id,
            name: self.name,
            parent: self.parent,
            translation: self.translation,
            scale: self.scale,
            render_layer_mask: zircon_runtime::scene::default_render_layer_mask(),
            native_fields: Vec::new(),
            plugin_components: Vec::new(),
        }
    }
}
