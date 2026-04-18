use crate::core::editor_event::{EditorEventEffect, EditorEventRecord};
use crate::{EditorEvent, LayoutCommand, MenuAction};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SlintDispatchEffects {
    pub presentation_dirty: bool,
    pub layout_dirty: bool,
    pub render_dirty: bool,
    pub active_layout_preset_name: Option<String>,
    pub present_welcome_surface: bool,
    pub sync_asset_workspace: bool,
    pub refresh_asset_details: bool,
    pub refresh_visible_asset_previews: bool,
    pub import_model_requested: bool,
    pub reset_active_layout_preset: bool,
}

pub(crate) fn apply_record_effects(target: &mut SlintDispatchEffects, record: &EditorEventRecord) {
    for effect in &record.effects {
        match effect {
            EditorEventEffect::PresentationChanged => {
                target.presentation_dirty = true;
            }
            EditorEventEffect::LayoutChanged => {
                target.layout_dirty = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::RenderChanged => {
                target.render_dirty = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::PresentWelcomeRequested => {
                target.present_welcome_surface = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::ProjectOpenRequested => {
                target.sync_asset_workspace = true;
                target.reset_active_layout_preset = true;
                target.layout_dirty = true;
                target.render_dirty = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::ProjectSaveRequested => {
                target.sync_asset_workspace = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::AssetDetailsRefreshRequested => {
                target.refresh_asset_details = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::AssetPreviewRefreshRequested => {
                target.refresh_visible_asset_previews = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::ImportModelRequested => {
                target.import_model_requested = true;
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
