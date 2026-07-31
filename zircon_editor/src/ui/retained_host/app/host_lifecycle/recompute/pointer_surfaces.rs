use super::super::super::RetainedEditorHost;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

impl RetainedEditorHost {
    pub(super) fn sync_recompute_pointer_surfaces(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        preset_names: &[String],
    ) {
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_pointer_surfaces");
        self.sync_menu_pointer_layout(model, chrome, preset_names);
        self.sync_welcome_recent_pointer_layout(chrome);
        let filtered_hierarchy_entries = self.filtered_hierarchy_entries(&chrome.scene_entries);
        let hierarchy_entries = filtered_hierarchy_entries
            .as_deref()
            .unwrap_or(&chrome.scene_entries);
        self.sync_hierarchy_pointer_layout(hierarchy_entries);
        self.sync_detail_pointer_layouts(chrome);
        self.sync_asset_pointer_layouts(chrome);
    }
}
