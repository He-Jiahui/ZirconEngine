use crate::ui::workbench::snapshot::InspectorSnapshot;

use super::PreviewInspector;

impl PreviewInspector {
    pub(crate) fn into_snapshot(self) -> InspectorSnapshot {
        InspectorSnapshot {
            id: self.id,
            name: self.name,
            parent: self.parent,
            translation: self.translation,
            scale: self.scale,
            render_layer_mask: zircon_runtime::scene::default_render_layer_mask(),
            plugin_components: Vec::new(),
        }
    }
}
