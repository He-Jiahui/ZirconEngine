use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventRecord, LayoutCommand, MenuAction,
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
    pub refresh_asset_details: bool,
    pub refresh_visible_asset_previews: bool,
    pub import_model_requested: bool,
    pub reset_active_layout_preset: bool,
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
