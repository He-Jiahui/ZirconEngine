use crate::editor_event::{EditorEventEffect, EditorEventRecord};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SlintDispatchEffects {
    pub presentation_dirty: bool,
    pub layout_dirty: bool,
    pub render_dirty: bool,
    pub sync_asset_workspace: bool,
    pub reset_active_layout_preset: bool,
}

pub(crate) fn apply_record_effects(
    target: &mut SlintDispatchEffects,
    record: &EditorEventRecord,
) {
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
            EditorEventEffect::ProjectOpenRequested => {
                target.sync_asset_workspace = true;
                target.reset_active_layout_preset = true;
                target.layout_dirty = true;
                target.render_dirty = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::ProjectSaveRequested | EditorEventEffect::AssetWorkspaceChanged => {
                target.sync_asset_workspace = true;
                target.presentation_dirty = true;
            }
            EditorEventEffect::ReflectionChanged => {}
        }
    }
}
