use std::error::Error;
use std::sync::Arc;

use zircon_runtime_interface::math::UVec2;

use crate::core::gui_startup_request::EditorGuiStartupRequest;
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
        let mut startup_session =
            resolve_editor_startup_session(editor_manager.as_ref(), startup_request)?;
        let state =
            build_startup_state(editor_manager.as_ref(), &mut startup_session, viewport_size)?;
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
        Some(EditorGuiStartupRequest::Project { intent }) => {
            editor_manager.execute_project_launch_intent(intent)
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
        None => editor_manager.resolve_startup_session(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn editor_startup_leaves_play_backend_ownership_to_the_app_composition() {
        let source = include_str!("editor_host_startup.rs");
        let product_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("product startup source should precede its tests");

        assert!(!product_source.contains("ProcessPlayBackend"));
        assert!(!product_source.contains("set_play_backend"));
    }
}
