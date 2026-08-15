use super::super::super::RetainedEditorHost;
use crate::ui::layouts::views::scene_viewport_chrome;
use crate::ui::retained_host::ui::to_host_contract_scene_viewport_chrome;
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::workbench::model::StatusBarModel;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_viewport_chrome_projection(&mut self) {
        let chrome = self.runtime.chrome_snapshot();
        let status = StatusBarModel::from_chrome(&chrome);
        let viewport = to_host_contract_scene_viewport_chrome(&scene_viewport_chrome(
            &chrome.scene_viewport_settings,
        ));
        if self.ui.patch_scene_viewport_chrome(
            viewport,
            status.grid_text.as_str(),
            status.snap_text.as_str(),
        ) {
            self.record_paint_only_invalidation(HostInvalidationMask::PAINT_ONLY);
        }
    }
}
