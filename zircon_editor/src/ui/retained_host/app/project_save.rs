use crate::ui::retained_host::callback_dispatch;

use super::RetainedEditorHost;

const SAVE_PROJECT_ACTION: &str = "workbench.project.save";

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn save_project_scene(&mut self) -> Result<(), String> {
        let effects = callback_dispatch::dispatch_menu_action(&self.runtime, SAVE_PROJECT_ACTION)?;
        self.apply_dispatch_effects(effects);
        Ok(())
    }
}
