use super::super::super::RetainedEditorHost;
use crate::ui::layouts::views::scene_viewport_chrome;
use crate::ui::retained_host::ui::to_host_contract_scene_viewport_chrome;
use crate::ui::workbench::model::StatusBarModel;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_viewport_chrome_projection(&mut self) -> bool {
        let chrome = self.runtime.chrome_snapshot();
        let status = StatusBarModel::from_chrome(&chrome);
        let viewport = to_host_contract_scene_viewport_chrome(&scene_viewport_chrome(
            &chrome.scene_viewport_settings,
        ));
        let root_changed = self.ui.patch_scene_viewport_chrome(
            viewport.clone(),
            status.grid_text.as_str(),
            status.snap_text.as_str(),
        );
        if root_changed {
            let damage = self
                .ui
                .viewport_chrome_damage_frame()
                .unwrap_or_else(|| self.ui.get_host_window_bootstrap().shell_frame);
            self.ui.request_frame_update_region(damage);
        }
        let native_patch = self
            .native_window_presenters
            .patch_scene_viewport_chrome(&viewport);
        root_changed || !native_patch.presenter_ids.is_empty()
    }
}
