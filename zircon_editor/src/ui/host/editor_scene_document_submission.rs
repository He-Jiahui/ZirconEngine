use std::sync::Arc;

use zircon_runtime::scene::Scene;

use crate::core::document::{AuthoringSceneInstaller, SceneDocumentRouteResult, ScenePickerTicket};
use crate::core::project::{SceneCreateRequest, SceneOpenRequest};
use crate::ui::workbench::state::EditorState;

use super::{EditorHostEventController, EditorManager};

/// The only host adapter allowed to install a project-authorized scene into editor state.
struct EditorStateSceneInstaller<'a> {
    manager: Arc<EditorManager>,
    state: &'a mut EditorState,
    project_path: String,
}

impl AuthoringSceneInstaller for EditorStateSceneInstaller<'_> {
    type Error = String;

    fn install_scene(&mut self, scene: &Scene) -> Result<(), Self::Error> {
        let level = self
            .manager
            .create_runtime_level(scene.clone())
            .map_err(|error| error.to_string())?;
        self.state.replace_world(level, &self.project_path)
    }
}

impl EditorHostEventController {
    /// Commits a picker-selected scene open request through the project/document authorities.
    pub fn submit_scene_open_request(
        &self,
        ticket: ScenePickerTicket,
        request: SceneOpenRequest,
    ) -> Result<SceneDocumentRouteResult, String> {
        let project_path = ticket.project_root().to_string_lossy().into_owned();
        let result = {
            let mut shell = self.shell().lock();
            let manager = Arc::clone(&shell.manager);
            let mut installer = EditorStateSceneInstaller {
                manager: Arc::clone(&manager),
                state: &mut shell.state,
                project_path,
            };
            manager
                .open_scene_document(ticket, request, &mut installer)
                .map_err(|error| error.to_string())?
        };
        Ok(self.finish_scene_document_submission(result))
    }

    /// Commits a picker-confirmed scene creation request through the project/document authorities.
    pub fn submit_scene_create_request(
        &self,
        ticket: ScenePickerTicket,
        request: SceneCreateRequest,
    ) -> Result<SceneDocumentRouteResult, String> {
        let project_path = ticket.project_root().to_string_lossy().into_owned();
        let result = {
            let mut shell = self.shell().lock();
            let manager = Arc::clone(&shell.manager);
            let mut installer = EditorStateSceneInstaller {
                manager: Arc::clone(&manager),
                state: &mut shell.state,
                project_path,
            };
            manager
                .create_scene_document(ticket, request, &mut installer)
                .map_err(|error| error.to_string())?
        };
        Ok(self.finish_scene_document_submission(result))
    }

    /// Captures the active session before a UI picker is shown.
    pub fn begin_scene_picker(&self) -> Result<ScenePickerTicket, String> {
        let manager = { Arc::clone(&self.shell().lock().manager) };
        manager
            .scene_picker_ticket()
            .map_err(|error| error.to_string())
    }

    fn finish_scene_document_submission(
        &self,
        result: SceneDocumentRouteResult,
    ) -> SceneDocumentRouteResult {
        if matches!(result, SceneDocumentRouteResult::Activated(_)) {
            self.publish_scene_inspection_resync();
            self.refresh_workbench(
                crate::core::editor_message::EditorViewInvalidationMask::RENDER.union(
                    crate::core::editor_message::EditorViewInvalidationMask::PRESENTATION_DATA,
                ),
            );
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_scene_installer_creates_a_runtime_level_before_replacing_authoring_world() {
        let source = include_str!("editor_scene_document_submission.rs");
        let create_level = [
            "self",
            "            .manager",
            "            .create_runtime_level(scene.clone())",
        ]
        .join("\n");
        let legacy_scene_install = [
            "self.state.replace_world(",
            "scene.clone(), &self.project_path)",
        ]
        .concat();

        assert!(source.contains(&create_level));
        assert!(source.contains("self.state.replace_world(level, &self.project_path)"));
        assert!(!source.contains(&legacy_scene_install));
    }
}
