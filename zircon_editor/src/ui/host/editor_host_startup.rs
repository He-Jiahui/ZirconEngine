use std::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::sync::Arc;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime_interface::math::UVec2;

use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::play::ProcessPlayBackend;
use crate::core::project::NewProjectDraft;
use crate::ui::retained_host::build_startup_state;
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};
use crate::ui::workbench::state::EditorState;

use super::{EditorError, EditorHostEventController, EditorManager};

/// Product startup state shared by the native host and application compositions.
pub struct EditorHostStartupSession {
    startup_session: EditorStartupSessionDocument,
    controller: EditorHostEventController,
}

impl EditorHostStartupSession {
    pub fn open(
        editor_manager: Arc<EditorManager>,
        startup_request: Option<EditorGuiStartupRequest>,
        viewport_size: UVec2,
    ) -> Result<Self, Box<dyn Error>> {
        Self::open_with_prepared_project(editor_manager, startup_request, None, viewport_size)
    }

    pub fn open_with_prepared_project(
        editor_manager: Arc<EditorManager>,
        startup_request: Option<EditorGuiStartupRequest>,
        prepared_project: Option<ProjectManager>,
        viewport_size: UVec2,
    ) -> Result<Self, Box<dyn Error>> {
        let startup_session = match prepared_project {
            Some(project) => editor_manager.open_prepared_project_and_remember(project)?,
            None => resolve_editor_startup_session(editor_manager.as_ref(), startup_request)?,
        };
        let state = build_startup_state(editor_manager.as_ref(), &startup_session, viewport_size)?;
        Self::from_parts(startup_session, state, editor_manager)
    }

    pub fn controller(&self) -> &EditorHostEventController {
        &self.controller
    }

    pub fn startup_session(&self) -> &EditorStartupSessionDocument {
        &self.startup_session
    }

    pub(crate) fn into_parts(self) -> (EditorStartupSessionDocument, EditorHostEventController) {
        (self.startup_session, self.controller)
    }

    pub(crate) fn from_parts(
        startup_session: EditorStartupSessionDocument,
        state: EditorState,
        editor_manager: Arc<EditorManager>,
    ) -> Result<Self, Box<dyn Error>> {
        let controller = EditorHostEventController::new(state, editor_manager);
        let backend = ProcessPlayBackend::for_current_install().map_err(|error| {
            IoError::new(
                ErrorKind::Other,
                format!("failed to configure the runtime play backend: {error}"),
            )
        })?;
        controller.set_play_backend(Arc::new(backend));

        Ok(Self {
            startup_session,
            controller,
        })
    }
}

pub(crate) fn resolve_editor_startup_session(
    editor_manager: &EditorManager,
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<EditorStartupSessionDocument, EditorError> {
    match startup_request {
        Some(EditorGuiStartupRequest::OpenProject { project_path }) => {
            editor_manager.open_project_and_remember(project_path)
        }
        Some(EditorGuiStartupRequest::OpenBuiltinView { descriptor_id }) => {
            Ok(EditorStartupSessionDocument {
                mode: EditorSessionMode::Welcome,
                project: None,
                open_builtin_view: Some(descriptor_id.clone()),
                recent_projects: Vec::new(),
                draft: NewProjectDraft::renderable_empty_default(),
                creation_validation: "Checking project location...".to_string(),
                can_open_existing: false,
                status_message: format!("Opened {descriptor_id}"),
            })
        }
        Some(EditorGuiStartupRequest::CreateProject(draft)) => {
            editor_manager.create_project_and_open(draft)
        }
        None => editor_manager.resolve_startup_session(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn product_startup_installs_the_process_play_backend() {
        let source = include_str!("editor_host_startup.rs");
        let product_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("product startup source should precede its tests");

        let backend = product_source
            .find("ProcessPlayBackend::for_current_install")
            .expect("product startup should construct the process play backend");
        let install = product_source
            .find("controller.set_play_backend")
            .expect("product startup should install the process play backend");

        assert!(backend < install);
    }
}
