use std::sync::Arc;

use crate::core::document::{
    ActiveSceneDocumentIdentity, ActiveSceneReloader, AuthoringSceneInstaller,
    SceneDocumentReloadCoordinator, SceneDocumentReloadError, SceneDocumentReloadOutcome,
    SceneDocumentRouteResult, ScenePickerTicket,
};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::project::{ProjectSceneDocument, SceneCreateRequest, SceneOpenRequest};
use crate::ui::workbench::state::{EditorState, EditorStateOperationError};
use zircon_runtime::asset::pipeline::manager::{
    ProjectAssetGenerationToken, ProjectAssetManager, ProjectGenerationCommitOutcome,
};

use super::{EditorHostEventController, EditorManager};

/// The only host adapter allowed to install a project-authorized scene into editor state.
struct EditorStateSceneInstaller<'a> {
    manager: Arc<EditorManager>,
    state: &'a mut EditorState,
    project_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparedActiveSceneReloadOutcome {
    Reloaded,
    Superseded,
    Conflict,
    ProjectGenerationSuperseded { newer_same_project_generation: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparedActiveSceneReloadDirtyPolicy {
    Reject,
    Discard,
}

#[derive(Debug)]
enum PreparedActiveSceneReloadError {
    State(EditorStateOperationError),
    ProjectGenerationSuperseded { newer_same_project_generation: bool },
}

impl std::fmt::Display for PreparedActiveSceneReloadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(error) => write!(formatter, "{error}"),
            Self::ProjectGenerationSuperseded {
                newer_same_project_generation,
            } => write!(
                formatter,
                "active scene reload project generation was superseded (newer same project: {newer_same_project_generation})"
            ),
        }
    }
}

struct EditorStateActiveSceneReloader<'a> {
    state: &'a mut EditorState,
    project_asset_manager: &'a ProjectAssetManager,
    generation: &'a ProjectAssetGenerationToken,
    authoring_world: Option<AuthoringWorldSeed>,
    dirty_policy: PreparedActiveSceneReloadDirtyPolicy,
}

impl ActiveSceneReloader for EditorStateActiveSceneReloader<'_> {
    type Error = PreparedActiveSceneReloadError;

    fn prepare_active_scene_reload(&mut self) -> Result<(), Self::Error> {
        if self.dirty_policy == PreparedActiveSceneReloadDirtyPolicy::Discard {
            return Ok(());
        }
        self.state
            .prepare_scene_transition()
            .map_err(PreparedActiveSceneReloadError::State)
    }

    fn install_active_scene_reload(&mut self) -> Result<(), Self::Error> {
        let authoring_world = self
            .authoring_world
            .take()
            .expect("a prepared scene reload installs at most once");
        let commit = || {
            self.state
                .reload_active_scene_world(authoring_world)
                .map_err(PreparedActiveSceneReloadError::State)
        };
        match self
            .project_asset_manager
            .commit_if_project_generation(self.generation, commit)
        {
            ProjectGenerationCommitOutcome::Committed(result) => result,
            ProjectGenerationCommitOutcome::Superseded {
                newer_same_project_generation,
            } => Err(
                PreparedActiveSceneReloadError::ProjectGenerationSuperseded {
                    newer_same_project_generation,
                },
            ),
        }
    }
}

impl AuthoringSceneInstaller for EditorStateSceneInstaller<'_> {
    type Error = String;

    fn prepare_scene_transition(&mut self) -> Result<(), Self::Error> {
        self.state
            .prepare_scene_transition()
            .map_err(|error| error.to_string())
    }

    fn install_scene(&mut self, document: &ProjectSceneDocument) -> Result<(), Self::Error> {
        let authoring_world = self
            .manager
            .prepare_authoring_world(document.world().clone())
            .map_err(|error| error.to_string())?;
        self.state
            .replace_world(authoring_world, &self.project_path)
            .map_err(|error| error.to_string())
    }
}

impl EditorHostEventController {
    pub(crate) fn commit_prepared_active_scene_reload(
        &self,
        project_asset_manager: &ProjectAssetManager,
        generation: &ProjectAssetGenerationToken,
        identity: ActiveSceneDocumentIdentity,
        authoring_world: AuthoringWorldSeed,
        dirty_policy: PreparedActiveSceneReloadDirtyPolicy,
    ) -> Result<PreparedActiveSceneReloadOutcome, String> {
        let result = {
            let mut shell = self.shell().lock();
            let manager = Arc::clone(&shell.manager);
            let mut reloader = EditorStateActiveSceneReloader {
                state: &mut shell.state,
                project_asset_manager,
                generation,
                authoring_world: Some(authoring_world),
                dirty_policy,
            };
            SceneDocumentReloadCoordinator::new(&manager.document_lifecycle)
                .reload(&identity, &mut reloader)
        };
        let outcome = match result {
            Ok(SceneDocumentReloadOutcome::Reloaded { .. }) => {
                PreparedActiveSceneReloadOutcome::Reloaded
            }
            Ok(SceneDocumentReloadOutcome::Superseded) => {
                PreparedActiveSceneReloadOutcome::Superseded
            }
            Err(SceneDocumentReloadError::Transition(PreparedActiveSceneReloadError::State(
                EditorStateOperationError::SceneTransitionDirty,
            ))) => PreparedActiveSceneReloadOutcome::Conflict,
            Err(SceneDocumentReloadError::Install(
                PreparedActiveSceneReloadError::ProjectGenerationSuperseded {
                    newer_same_project_generation,
                },
            )) => PreparedActiveSceneReloadOutcome::ProjectGenerationSuperseded {
                newer_same_project_generation,
            },
            Err(error) => return Err(error.to_string()),
        };
        if outcome == PreparedActiveSceneReloadOutcome::Reloaded {
            self.publish_scene_inspection_resync();
            self.refresh_workbench(
                crate::core::editor_message::EditorViewInvalidationMask::RENDER.union(
                    crate::core::editor_message::EditorViewInvalidationMask::PRESENTATION_DATA,
                ),
            );
        }
        Ok(outcome)
    }

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
        let document = match &result {
            SceneDocumentRouteResult::Activated(activation) => activation.activation.document,
            SceneDocumentRouteResult::AlreadyActive { document } => *document,
        };
        self.shell().lock().state.bind_scene_document(document);
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
    fn project_scene_installer_prepares_an_authoring_seed_before_replacing_authoring_world() {
        let source = include_str!("editor_scene_document_submission.rs");
        let prepare_seed = [
            "self",
            "            .manager",
            "            .prepare_authoring_world(document.world().clone())",
        ]
        .join("\n");
        let raw_level_install = [
            "self.state.replace_world(",
            "document.world().clone(), &self.project_path)",
        ]
        .concat();

        assert!(source.contains(&prepare_seed));
        let install_authoring_world = [
            "self.state",
            "            .replace_world(authoring_world, &self.project_path)",
        ]
        .join("\n");
        assert!(source.contains(".prepare_scene_transition()"));
        assert!(source.contains(&install_authoring_world));
        assert!(!source.contains(".create_runtime_level(scene.clone())"));
        assert!(!source.contains(&raw_level_install));
    }

    #[test]
    fn authoring_seed_preparation_does_not_reexpose_the_runtime_level_through_manager_project() {
        let source = include_str!("editor_manager_project.rs");

        assert!(source.contains(".prepare_authoring_world(scene)"));
        assert!(!source.contains(".create_runtime_level(scene)"));
    }

    #[test]
    fn scene_submission_binds_lifecycle_document_for_new_and_already_active_routes() {
        let source = include_str!("editor_scene_document_submission.rs");

        assert!(source.contains(
            "SceneDocumentRouteResult::Activated(activation) => activation.activation.document"
        ));
        assert!(
            source.contains("SceneDocumentRouteResult::AlreadyActive { document } => *document")
        );
        assert!(source.contains(".state.bind_scene_document(document)"));
    }
}
