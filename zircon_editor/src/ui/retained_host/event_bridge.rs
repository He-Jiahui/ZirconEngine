use std::collections::BTreeSet;
use std::time::Duration;

use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventRecord, LayoutCommand, MenuAction,
};
use crate::core::notifications::{
    NotificationId, NotificationSource, ToastNotification, ToastSeverity,
};
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct HostShellContentScope {
    pub(crate) slot: ActivityDrawerSlot,
    pub(crate) instance_id: ViewInstanceId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AssetRelocationRequest {
    pub(crate) asset_uuid: String,
    pub(crate) target_locator: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AssetDeletionRequest {
    pub(crate) asset_uuid: String,
}

impl HostShellContentScope {
    pub(crate) fn new(slot: ActivityDrawerSlot, instance_id: ViewInstanceId) -> Self {
        Self { slot, instance_id }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiHostEventEffects {
    pub dirty_domains: HostInvalidationMask,
    shell_content_scopes: BTreeSet<HostShellContentScope>,
    pub sync_viewport_chrome: bool,
    pub presentation_dirty: bool,
    pub layout_dirty: bool,
    pub render_dirty: bool,
    pub active_layout_preset_name: Option<String>,
    pub present_welcome_surface: bool,
    pub sync_asset_workspace: bool,
    pub save_all_documents: bool,
    pub close_active_project: bool,
    pub refresh_asset_details: bool,
    pub refresh_visible_asset_previews: bool,
    pub import_model_requested: bool,
    pub asset_deletion_requested: Option<AssetDeletionRequest>,
    pub asset_relocation_requested: Option<AssetRelocationRequest>,
    pub reset_active_layout_preset: bool,
    pub open_command_palette_requested: bool,
    pub open_settings_window_requested: bool,
    pub open_scene_picker_requested: bool,
    pub create_scene_picker_requested: bool,
    pub toast_notifications: Vec<ToastNotification>,
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

    pub(crate) fn request_workbench_projection(&mut self) {
        self.merge_dirty_domains(HostInvalidationMask::WORKBENCH_PROJECTION);
    }

    pub(crate) fn reuse_layout_for_shell_content(&mut self, scope: HostShellContentScope) {
        self.dirty_domains.remove(HostInvalidationMask::LAYOUT);
        self.dirty_domains
            .remove(HostInvalidationMask::PRESENTATION_DATA);
        self.dirty_domains
            .insert(HostInvalidationMask::SHELL_CONTENT);
        self.layout_dirty = self.dirty_domains.requires_layout();
        self.presentation_dirty = true;
        self.shell_content_scopes.insert(scope);
    }

    pub(crate) fn shell_content_scope(&self) -> Option<HostShellContentScope> {
        (self.shell_content_scopes.len() == 1)
            .then(|| self.shell_content_scopes.iter().next().cloned())
            .flatten()
    }

    pub(crate) fn merge_shell_content_scope_state_from(&mut self, source: &Self) {
        let target_is_scoped_or_paint_only = !self.dirty_domains().requires_host_recompute()
            || !self.shell_content_scopes.is_empty();
        let source_is_scoped_or_paint_only = !source.dirty_domains().requires_host_recompute()
            || !source.shell_content_scopes.is_empty();
        if target_is_scoped_or_paint_only && source_is_scoped_or_paint_only {
            self.shell_content_scopes
                .extend(source.shell_content_scopes.iter().cloned());
        } else {
            self.shell_content_scopes.clear();
        }
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

    pub(crate) fn is_viewport_resize_recompute_compatible(&self) -> bool {
        let allowed = HostInvalidationMask::PRESENTATION_DATA
            .union(HostInvalidationMask::RENDER)
            .union(HostInvalidationMask::PAINT_ONLY);
        let dirty = self.dirty_domains();
        dirty.contains(HostInvalidationMask::PRESENTATION_DATA)
            && dirty.contains(HostInvalidationMask::RENDER)
            && dirty.intersection(allowed) == dirty
            && self.shell_content_scopes.is_empty()
            && !self.sync_viewport_chrome
            && self.active_layout_preset_name.is_none()
            && !self.present_welcome_surface
            && !self.sync_asset_workspace
            && !self.save_all_documents
            && !self.close_active_project
            && !self.refresh_asset_details
            && !self.refresh_visible_asset_previews
            && !self.import_model_requested
            && self.asset_deletion_requested.is_none()
            && self.asset_relocation_requested.is_none()
            && !self.reset_active_layout_preset
            && !self.open_command_palette_requested
            && !self.open_settings_window_requested
            && !self.open_scene_picker_requested
            && !self.create_scene_picker_requested
            && self.toast_notifications.is_empty()
    }

    pub(crate) fn merge_dirty_domains(&mut self, dirty_domains: HostInvalidationMask) {
        let shell_content_compatible = dirty_domains.contains(HostInvalidationMask::SHELL_CONTENT)
            && dirty_domains.intersection(
                HostInvalidationMask::SHELL_CONTENT
                    .union(HostInvalidationMask::PRESENTATION_DATA)
                    .union(HostInvalidationMask::PAINT_ONLY)
                    .union(HostInvalidationMask::POINTER_HOVER)
                    .union(HostInvalidationMask::VIEWPORT_IMAGE),
            ) == dirty_domains;
        if dirty_domains.requires_host_recompute() && !shell_content_compatible {
            self.shell_content_scopes.clear();
        }
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

    let notifications = toast_notifications_for_record(record);
    if !notifications.is_empty() {
        target.toast_notifications.extend(notifications);
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
            EditorEventEffect::DocumentSaveAllRequested => {
                target.save_all_documents = true;
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
            EditorEventEffect::AssetRelocationRequested {
                asset_uuid,
                target_locator,
            } => {
                target.asset_relocation_requested = Some(AssetRelocationRequest {
                    asset_uuid: asset_uuid.clone(),
                    target_locator: target_locator.clone(),
                });
            }
            EditorEventEffect::AssetDeletionRequested { asset_uuid } => {
                target.asset_deletion_requested = Some(AssetDeletionRequest {
                    asset_uuid: asset_uuid.clone(),
                });
            }
            EditorEventEffect::CommandPaletteOpenRequested => {
                target.open_command_palette_requested = true;
                target.request_presentation();
            }
            EditorEventEffect::SettingsWindowOpenRequested => {
                target.open_settings_window_requested = true;
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

    if matches!(
        &record.event,
        EditorEvent::Viewport(event) if event.changes_chrome_projection()
    ) {
        target.sync_viewport_chrome = true;
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

const DEFAULT_TOAST_LIFETIME: Duration = Duration::from_millis(3_500);
const IMPORT_TOAST_LIFETIME: Duration = Duration::from_secs(4);
const ERROR_TOAST_LIFETIME: Duration = Duration::from_secs(7);
const RETAINED_HOST_NOTIFICATION_SOURCE: &str = "editor.retained_host";

fn toast_notifications_for_record(record: &EditorEventRecord) -> Vec<ToastNotification> {
    if let Some(error) = record
        .result
        .error
        .as_deref()
        .map(str::trim)
        .filter(|error| !error.is_empty())
    {
        return toast_for_record(
            record,
            "command-failed",
            ToastSeverity::Error,
            "editor.notification.command_failed.title",
            ToastNotification::bounded_message(error, "The editor command could not complete."),
            ERROR_TOAST_LIFETIME,
        )
        .into_iter()
        .collect();
    }

    record
        .effects
        .iter()
        .filter_map(|effect| match effect {
            EditorEventEffect::ProjectOpenRequested => toast_for_record(
                record,
                "project-open",
                ToastSeverity::Success,
                "editor.notification.project_opened.title",
                "editor.notification.project_opened.message",
                DEFAULT_TOAST_LIFETIME,
            ),
            EditorEventEffect::ProjectSaveRequested => toast_for_record(
                record,
                "project-save",
                ToastSeverity::Success,
                "editor.notification.project_saved.title",
                "editor.notification.project_saved.message",
                DEFAULT_TOAST_LIFETIME,
            ),
            EditorEventEffect::ImportModelRequested => toast_for_record(
                record,
                "import-model",
                ToastSeverity::Info,
                "editor.notification.import_model.title",
                "editor.notification.import_model.message",
                IMPORT_TOAST_LIFETIME,
            ),
            _ => None,
        })
        .collect()
}

fn toast_for_record(
    record: &EditorEventRecord,
    suffix: &str,
    severity: ToastSeverity,
    title_key: &str,
    message_key: impl Into<String>,
    lifetime: Duration,
) -> Option<ToastNotification> {
    let id =
        NotificationId::parse(format!("editor.event.{}.{}", record.sequence.0, suffix)).ok()?;
    let source = NotificationSource::builtin(RETAINED_HOST_NOTIFICATION_SOURCE).ok()?;
    ToastNotification::new(id, source, severity, title_key, message_key, lifetime).ok()
}

#[cfg(test)]
mod tests {
    use crate::core::editor_event::{
        EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
        EditorEventSequence, EditorEventSource, EditorEventUndoPolicy, MenuAction,
    };

    use super::{apply_record_effects, UiHostEventEffects};

    #[test]
    fn viewport_resize_projection_requires_presentation_and_render_only() {
        let mut effects = UiHostEventEffects::default();
        effects.request_presentation();
        effects.request_render();

        assert!(effects.is_viewport_resize_recompute_compatible());

        effects.request_layout();
        assert!(!effects.is_viewport_resize_recompute_compatible());
    }

    #[test]
    fn viewport_resize_projection_rejects_an_incomplete_effect_set() {
        let mut presentation_only = UiHostEventEffects::default();
        presentation_only.request_presentation();
        assert!(!presentation_only.is_viewport_resize_recompute_compatible());

        let mut render_only = UiHostEventEffects::default();
        render_only.request_render();
        assert!(!render_only.is_viewport_resize_recompute_compatible());
    }

    #[test]
    fn workbench_projection_effect_preserves_coalesced_render_without_global_presentation() {
        let mut effects = UiHostEventEffects::default();
        effects.request_paint_only();
        effects.request_render();
        effects.request_workbench_projection();

        let dirty = effects.dirty_domains();
        assert!(dirty.contains(crate::ui::retained_host::HostInvalidationMask::PAINT_ONLY));
        assert!(dirty.contains(crate::ui::retained_host::HostInvalidationMask::RENDER));
        assert!(
            dirty.contains(crate::ui::retained_host::HostInvalidationMask::WORKBENCH_PROJECTION)
        );
        assert!(dirty.requires_host_recompute());
        assert!(!dirty.requires_presentation());
    }

    #[test]
    fn stable_shell_content_replaces_generic_layout_invalidation() {
        let mut effects = UiHostEventEffects::default();
        effects.request_layout();
        effects.request_presentation();

        let scope = crate::ui::retained_host::HostShellContentScope::new(
            crate::ui::workbench::layout::ActivityDrawerSlot::LeftBottom,
            crate::ui::workbench::view::ViewInstanceId::new("editor.module_plugins#main"),
        );
        effects.reuse_layout_for_shell_content(scope.clone());

        assert!(!effects.dirty_domains().requires_layout());
        assert!(effects.dirty_domains().requires_presentation());
        assert!(effects
            .dirty_domains()
            .contains(crate::ui::retained_host::HostInvalidationMask::SHELL_CONTENT));
        assert!(!effects.layout_dirty);
        assert!(effects.presentation_dirty);
        assert_eq!(effects.shell_content_scope(), Some(scope));
    }

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

    #[test]
    fn asset_relocation_effect_preserves_background_request_payload() {
        let asset_uuid = "00112233-4455-6677-8899-aabbccddeeff".to_owned();
        let target_locator = "res://environment/cube.zmodel".to_owned();
        let record = EditorEventRecord {
            event_id: EditorEventId::new(2),
            sequence: EditorEventSequence::new(2),
            source: EditorEventSource::RetainedHost,
            event: EditorEvent::Asset(crate::core::editor_event::EditorAssetEvent::RelocateAsset {
                asset_uuid: asset_uuid.clone(),
                target_locator: target_locator.clone(),
            }),
            binding_path: Some("AssetTree/RelocateAsset".to_owned()),
            operation_id: None,
            operation_display_name: None,
            operation_arguments: None,
            operation_group: None,
            transaction_id: None,
            save_generation: None,
            effects: vec![EditorEventEffect::AssetRelocationRequested {
                asset_uuid: asset_uuid.clone(),
                target_locator: target_locator.clone(),
            }],
            undo_policy: EditorEventUndoPolicy::NonUndoable,
            before_revision: 0,
            after_revision: 0,
            result: EditorEventResult::success(serde_json::json!({ "changed": false })),
        };
        let mut effects = UiHostEventEffects::default();

        apply_record_effects(&mut effects, &record);

        assert_eq!(
            effects.asset_relocation_requested,
            Some(super::AssetRelocationRequest {
                asset_uuid,
                target_locator,
            })
        );
    }

    #[test]
    fn asset_deletion_effect_preserves_background_request_payload() {
        let asset_uuid = "00112233-4455-6677-8899-aabbccddeeff".to_owned();
        let record = EditorEventRecord {
            event_id: EditorEventId::new(3),
            sequence: EditorEventSequence::new(3),
            source: EditorEventSource::RetainedHost,
            event: EditorEvent::Asset(crate::core::editor_event::EditorAssetEvent::DeleteAsset {
                asset_uuid: asset_uuid.clone(),
            }),
            binding_path: Some("AssetContextMenu/DeleteAsset".to_owned()),
            operation_id: None,
            operation_display_name: None,
            operation_arguments: None,
            operation_group: None,
            transaction_id: None,
            save_generation: None,
            effects: vec![EditorEventEffect::AssetDeletionRequested {
                asset_uuid: asset_uuid.clone(),
            }],
            undo_policy: EditorEventUndoPolicy::NonUndoable,
            before_revision: 0,
            after_revision: 0,
            result: EditorEventResult::success(serde_json::json!({ "changed": false })),
        };
        let mut effects = UiHostEventEffects::default();

        apply_record_effects(&mut effects, &record);

        assert_eq!(
            effects.asset_deletion_requested,
            Some(super::AssetDeletionRequest { asset_uuid })
        );
    }
}
