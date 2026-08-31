use std::sync::Arc;

use zircon_runtime::asset::AssetUri;

use crate::core::project::SceneOpenRequest;
use crate::ui::binding::EditorUiEventKind;
use crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::scene_picker_session::{ScenePickerMode, ScenePickerSubmission};
use super::{HostInvalidationMask, RetainedEditorHost};

const SCENE_PICKER_CONTROL_ID: &str = "WorkbenchCommandPalette";
const QUERY_BINDING_ID: &str = "CommandPalette/QueryChanged";
const WINDOW_BINDING_ID: &str = "CommandPalette/WindowRequested";
const COMMIT_BINDING_ID: &str = "CommandPalette/Commit";

impl RetainedEditorHost {
    pub(super) fn open_startup_scene(&mut self, scene_uri: AssetUri) -> Result<(), String> {
        let scene_uri_text = scene_uri.to_string();
        let ticket = self.runtime.begin_scene_picker()?;
        let request = SceneOpenRequest::new(scene_uri);
        self.runtime.submit_scene_open_request(ticket, request)?;
        self.set_status_line(format!("Opened scene {scene_uri_text}"));
        Ok(())
    }

    pub(super) fn open_workbench_scene_picker(&mut self, mode: ScenePickerMode) {
        let ticket = match self.runtime.begin_scene_picker() {
            Ok(ticket) => ticket,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        let catalog: Arc<EditorAssetCatalogGeneration> =
            match self.editor_asset_manager_at_use_point() {
                Ok(manager) => manager.catalog_snapshot(),
                Err(error) => {
                    self.set_status_line(error.to_string());
                    return;
                }
            };
        let session = super::scene_picker_session::ScenePickerSession::new(ticket, mode, &catalog);
        let state = session.command_palette_state("", 0, false);
        match self
            .workbench_window_bridge
            .open_command_palette_with_chrome(
                state,
                mode.command_source(),
                mode.placeholder(),
                mode.empty_text(),
                mode.accessibility_label(),
                mode.empty_text(),
            ) {
            Ok(true) => {
                self.scene_picker_session = Some(session);
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            }
            Ok(false) => self.set_status_line("Scene picker surface is unavailable"),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_workbench_scene_picker_query_edited(
        &mut self,
        control_id: &str,
        binding_id: &str,
        query: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.is_scene_picker_binding(control_id, binding_id, EditorUiEventKind::Change)
            || self.active_scene_picker_mode().is_none()
        {
            return None;
        }
        let Some(session) = self.scene_picker_session.as_ref() else {
            return Some(Err(
                "scene picker session is no longer available".to_string()
            ));
        };
        let state = session.command_palette_state(query, 0, false);
        let updated = self
            .workbench_window_bridge
            .update_command_palette_query(state)
            .map_err(|error| error.to_string());
        Some(updated.map(paint_only_effects))
    }

    pub(in crate::ui::retained_host::app) fn dispatch_workbench_scene_picker_window_requested(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.is_scene_picker_binding(control_id, binding_id, EditorUiEventKind::Change)
            || self.active_scene_picker_mode().is_none()
        {
            return None;
        }
        let Some((current_offset, target_offset, focus_last, request_query)) =
            parse_window_request(value)
        else {
            return Some(Err(format!(
                "invalid scene picker window request `{value}`"
            )));
        };
        if self.workbench_window_bridge.command_palette_window_offset() != Some(current_offset)
            || self
                .workbench_window_bridge
                .command_palette_query()
                .as_str()
                != request_query
        {
            return Some(Ok(UiHostEventEffects::default()));
        }
        let Some(session) = self.scene_picker_session.as_ref() else {
            return Some(Err(
                "scene picker session is no longer available".to_string()
            ));
        };
        let state = session.command_palette_state(request_query, target_offset, focus_last);
        let updated = self
            .workbench_window_bridge
            .update_command_palette_query(state)
            .map_err(|error| error.to_string());
        Some(updated.map(paint_only_effects))
    }

    pub(in crate::ui::retained_host::app) fn dispatch_workbench_scene_picker_committed(
        &mut self,
        control_id: &str,
        binding_id: &str,
        command_id: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.is_scene_picker_binding(control_id, binding_id, EditorUiEventKind::Submit) {
            return None;
        }
        let Some(mode) = self.active_scene_picker_mode() else {
            return None;
        };
        let query = self.workbench_window_bridge.command_palette_query();
        let Some(window_offset) = self.workbench_window_bridge.command_palette_window_offset()
        else {
            let error = "scene picker result window is no longer available".to_string();
            self.set_status_line(error.clone());
            return Some(Err(error));
        };
        let Some(session) = self.scene_picker_session.as_ref() else {
            return Some(Err(
                "scene picker session is no longer available".to_string()
            ));
        };
        let submission = match session.submission(command_id, &query, window_offset) {
            Ok(submission) => submission,
            Err(error) => {
                self.set_status_line(error.clone());
                return Some(Err(error));
            }
        };

        let result = match submission {
            ScenePickerSubmission::Open { ticket, request } => {
                let scene_uri = request.scene_uri().to_string();
                self.runtime
                    .submit_scene_open_request(ticket, request)
                    .map(|_| format!("Opened scene {scene_uri}"))
            }
            ScenePickerSubmission::Create { ticket, request } => {
                let scene_uri = request.scene_uri().to_string();
                self.runtime
                    .submit_scene_create_request(ticket, request)
                    .map(|_| format!("Created scene {scene_uri}"))
            }
        };
        match result {
            Ok(status) => {
                self.scene_picker_session = None;
                if let Err(error) = self.workbench_window_bridge.close_command_palette() {
                    self.set_status_line(error.to_string());
                } else {
                    self.set_status_line(status);
                }
                let mut effects = UiHostEventEffects::default();
                effects.request_render_and_presentation();
                Some(Ok(effects))
            }
            Err(error) => {
                let message = format!(
                    "{} scene request failed: {error}",
                    scene_picker_action_name(mode)
                );
                self.set_status_line(message.clone());
                Some(Err(message))
            }
        }
    }

    fn active_scene_picker_mode(&self) -> Option<ScenePickerMode> {
        let source = self.workbench_window_bridge.command_palette_source();
        self.scene_picker_session
            .as_ref()
            .map(|session| session.mode())
            .filter(|mode| mode.command_source() == source)
    }

    fn is_scene_picker_binding(
        &self,
        control_id: &str,
        binding_id: &str,
        event_kind: EditorUiEventKind,
    ) -> bool {
        let expected_binding_id = match event_kind {
            EditorUiEventKind::Change => {
                if binding_id == QUERY_BINDING_ID || binding_id == WINDOW_BINDING_ID {
                    binding_id
                } else {
                    return false;
                }
            }
            EditorUiEventKind::Submit if binding_id == COMMIT_BINDING_ID => binding_id,
            _ => return false,
        };
        control_id == SCENE_PICKER_CONTROL_ID
            && self
                .workbench_window_bridge
                .binding_by_id(expected_binding_id)
                .is_some_and(|binding| binding.path().event_kind == event_kind)
    }
}

fn paint_only_effects(updated: bool) -> UiHostEventEffects {
    let mut effects = UiHostEventEffects::default();
    if updated {
        effects.request_paint_only();
    }
    effects
}

fn parse_window_request(value: &str) -> Option<(usize, usize, bool, &str)> {
    let mut fields = value.splitn(4, '|');
    let current_offset = fields.next()?.parse().ok()?;
    let target_offset = fields.next()?.parse().ok()?;
    let focus_last = match fields.next()? {
        "first" => false,
        "last" => true,
        _ => return None,
    };
    let query = fields.next()?;
    Some((current_offset, target_offset, focus_last, query))
}

fn scene_picker_action_name(mode: ScenePickerMode) -> &'static str {
    match mode {
        ScenePickerMode::Open => "Open",
        ScenePickerMode::Create => "Create",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn startup_scene_submission_reuses_the_document_route() {
        let source = include_str!("scene_picker_actions.rs");

        assert!(source.contains("fn open_startup_scene"));
        assert!(source.contains("self.runtime.begin_scene_picker()"));
        assert!(source.contains("SceneOpenRequest::new(scene_uri)"));
        assert!(source.contains(".submit_scene_open_request(ticket, request)"));
    }
}

#[cfg(test)]
#[path = "scene_picker_actions/borrowed_window_request_tests.rs"]
mod borrowed_window_request_tests;
