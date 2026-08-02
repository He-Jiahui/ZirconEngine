use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventRecord, LayoutCommand, MenuAction,
};
use crate::ui::retained_host::workbench_notifications::{
    workbench_notifications_for_record, WorkbenchNotification,
};
use crate::ui::retained_host::HostInvalidationMask;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiHostEventEffects {
    pub dirty_domains: HostInvalidationMask,
    pub presentation_dirty: bool,
    pub layout_dirty: bool,
    pub render_dirty: bool,
    pub active_layout_preset_name: Option<String>,
    pub present_welcome_surface: bool,
    pub sync_asset_workspace: bool,
    pub close_active_project: bool,
    pub refresh_asset_details: bool,
    pub refresh_visible_asset_previews: bool,
    pub import_model_requested: bool,
    pub reset_active_layout_preset: bool,
    pub open_command_palette_requested: bool,
    pub open_scene_picker_requested: bool,
    pub create_scene_picker_requested: bool,
    pub workbench_notifications: Vec<WorkbenchNotification>,
}

impl UiHostEventEffects {
    pub(crate) fn request_presentation(&mut self) {
        self.merge_dirty_domains(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(crate) fn request_layout(&mut self) {
        self.merge_dirty_domains(HostInvalidationMask::LAYOUT);
    }

    pub(crate) fn request_render(&mut self) {
        self.merge_dirty_domains(HostInvalidationMask::RENDER);
    }

    pub(crate) fn request_render_and_presentation(&mut self) {
        self.merge_dirty_domains(
            HostInvalidationMask::RENDER.union(HostInvalidationMask::PRESENTATION_DATA),
        );
    }

    pub(crate) fn request_paint_only(&mut self) {
        self.merge_dirty_domains(HostInvalidationMask::PAINT_ONLY);
    }

    pub(crate) fn dirty_domains(&self) -> HostInvalidationMask {
        self.dirty_domains
            .union(HostInvalidationMask::from_dirty_flags(
                self.layout_dirty,
                self.presentation_dirty,
                false,
                self.render_dirty,
            ))
    }

    pub(crate) fn merge_dirty_domains(&mut self, dirty_domains: HostInvalidationMask) {
        self.dirty_domains.insert(dirty_domains);
        if dirty_domains.requires_layout() {
            self.layout_dirty = true;
        }
        if dirty_domains.requires_presentation() || dirty_domains.requires_hit_test() {
            self.presentation_dirty = true;
        }
        if dirty_domains.requires_render() {
            self.render_dirty = true;
        }
    }
}

pub(crate) fn apply_record_effects(target: &mut UiHostEventEffects, record: &EditorEventRecord) {
    if record.operation_group.as_deref() == Some("MaterialComponentLab") {
        target.request_paint_only();
    }

    let notifications = workbench_notifications_for_record(record);
    if !notifications.is_empty() {
        target.workbench_notifications.extend(notifications);
        target.request_presentation();
    }

    for effect in &record.effects {
        match effect {
            EditorEventEffect::PresentationChanged => {
                target.request_presentation();
            }
            EditorEventEffect::LayoutChanged => {
                target.request_layout();
            }
            EditorEventEffect::RenderChanged => {
                target.request_render();
            }
            EditorEventEffect::PresentWelcomeRequested => {
                target.present_welcome_surface = true;
                target.request_presentation();
            }
            EditorEventEffect::ProjectOpenRequested => {
                target.sync_asset_workspace = true;
                target.reset_active_layout_preset = true;
                target.request_layout();
                target.request_render_and_presentation();
            }
            EditorEventEffect::ProjectSaveRequested => {
                target.sync_asset_workspace = true;
                target.request_presentation();
            }
            EditorEventEffect::ProjectCloseRequested => {
                target.close_active_project = true;
                target.sync_asset_workspace = true;
                target.request_presentation();
            }
            EditorEventEffect::AssetDetailsRefreshRequested => {
                target.refresh_asset_details = true;
                target.request_presentation();
            }
            EditorEventEffect::AssetPreviewRefreshRequested => {
                target.refresh_visible_asset_previews = true;
                target.request_paint_only();
            }
            EditorEventEffect::ImportModelRequested => {
                target.import_model_requested = true;
            }
            EditorEventEffect::CommandPaletteOpenRequested => {
                target.open_command_palette_requested = true;
                target.request_presentation();
            }
            EditorEventEffect::OpenScenePickerRequested => {
                target.open_scene_picker_requested = true;
                target.request_presentation();
            }
            EditorEventEffect::CreateScenePickerRequested => {
                target.create_scene_picker_requested = true;
                target.request_presentation();
            }
            EditorEventEffect::ReflectionChanged => {}
        }
    }

    match &record.event {
        EditorEvent::Layout(LayoutCommand::SavePreset { name })
        | EditorEvent::Layout(LayoutCommand::LoadPreset { name }) => {
            target.active_layout_preset_name = Some(name.clone());
        }
        EditorEvent::Layout(LayoutCommand::ResetToDefault)
        | EditorEvent::WorkbenchMenu(MenuAction::ResetLayout) => {
            target.reset_active_layout_preset = true;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::core::editor_event::{
        EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
        EditorEventSequence, EditorEventSource, EditorEventUndoPolicy, MenuAction,
    };

    use super::{apply_record_effects, UiHostEventEffects};

    #[test]
    fn project_close_effect_requests_retained_close_and_empty_asset_sync() {
        let record = EditorEventRecord {
            event_id: EditorEventId::new(1),
            sequence: EditorEventSequence::new(1),
            source: EditorEventSource::RetainedHost,
            event: EditorEvent::WorkbenchMenu(MenuAction::CloseProject),
            binding_path: None,
            operation_id: None,
            operation_display_name: None,
            operation_arguments: None,
            operation_group: None,
            transaction_id: None,
            save_generation: None,
            effects: vec![
                EditorEventEffect::ProjectCloseRequested,
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
            undo_policy: EditorEventUndoPolicy::FutureInverseEvent,
            before_revision: 0,
            after_revision: 0,
            result: EditorEventResult::success(serde_json::json!({
                "revision": 0,
                "changed": false,
            })),
        };
        let mut effects = UiHostEventEffects::default();

        apply_record_effects(&mut effects, &record);

        assert!(effects.close_active_project);
        assert!(effects.sync_asset_workspace);
        assert!(effects.presentation_dirty);
        assert!(!effects.layout_dirty);
        assert!(!effects.render_dirty);
    }
}
