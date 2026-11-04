use crate::ui::retained_host::callback_dispatch;

use super::RetainedEditorHost;

pub(super) struct SceneOverwriteSaveFailure {
    pub source_conflict: bool,
    pub persisted: bool,
    pub message: String,
}

const SAVE_PROJECT_ACTION: &str = "workbench.project.save";

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn save_project_scene_with_overwrite(
        &mut self,
        lease: crate::core::document::OverwriteLease,
    ) -> Result<(), SceneOverwriteSaveFailure> {
        let record = self
            .runtime
            .dispatch_authorized_scene_save(lease)
            .map_err(|error| {
                let source = match &error {
                    crate::ui::host::EditorEventDispatchError::Execution(source) => Some(source),
                    _ => None,
                };
                SceneOverwriteSaveFailure {
                    source_conflict: source.is_some_and(|source| source.is_scene_source_conflict()),
                    persisted: source
                        .is_some_and(|source| source.persisted_scene_receipt().is_some()),
                    message: error.to_string(),
                }
            })?;
        let mut effects = crate::ui::retained_host::event_bridge::UiHostEventEffects::default();
        crate::ui::retained_host::event_bridge::apply_record_effects(&mut effects, &record);
        self.apply_dispatch_effects(effects);
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn save_project_scene(&mut self) -> Result<(), String> {
        let effects = callback_dispatch::dispatch_menu_action(&self.runtime, SAVE_PROJECT_ACTION)?;
        self.apply_dispatch_effects(effects);
        Ok(())
    }
}
