use zircon_runtime::scene::Scene;

use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditCommandError, ExclusiveTransition, HistoryContextId};
use crate::ui::workbench::state::EditorState;

use super::{EditorSessionMode, WelcomePaneSnapshot};

impl EditorState {
    pub fn replace_world(
        &mut self,
        world: impl Into<AuthoringWorldSeed>,
        project_path: impl Into<String>,
    ) -> Result<(), String> {
        let project_path = project_path.into();
        self.with_exclusive_scene_transition("replace editor world", move |state, transition| {
            transition
                .clear_history_and_context::<CoreEditContext>(
                    HistoryContextId::Global,
                    "CoreEditContext",
                    |context| {
                        context.clear_scene()?;
                        state.world.replace(world).map_err(|error| {
                            EditCommandError::ExternalEffect {
                                source: Box::new(error),
                            }
                        })
                    },
                )
                .map_err(|error| error.to_string())?;
            state.project_path = project_path;
            state
                .viewport_controller
                .configure_project_settings(Path::new(&state.project_path));
            state.session_mode = EditorSessionMode::Project;
            state.project_open = true;
            state.welcome = WelcomePaneSnapshot::default();
            *state.viewport_controller.selection_mut() = Default::default();
            let _ = state
                .world
                .try_with_world(|scene| state.viewport_controller.reset_from_scene(Some(scene)));
            state.sync_selection_state();
            Ok(())
        })
    }

    pub fn clear_project(&mut self, welcome: WelcomePaneSnapshot) -> Result<(), String> {
        self.with_exclusive_scene_transition("clear editor project", move |state, transition| {
            transition
                .clear_history_and_context::<CoreEditContext>(
                    HistoryContextId::Global,
                    "CoreEditContext",
                    |context| {
                        context.clear_scene()?;
                        state
                            .world
                            .clear()
                            .map_err(|error| EditCommandError::ExternalEffect {
                                source: Box::new(error),
                            })
                    },
                )
                .map_err(|error| error.to_string())?;
            state.project_path.clear();
            state.viewport_controller.clear_project_settings();
            state.session_mode = EditorSessionMode::Welcome;
            state.project_open = false;
            state.welcome = welcome;
            *state.viewport_controller.selection_mut() = Default::default();
            state.viewport_controller.reset_from_scene(None);
            state.sync_selection_state();
            Ok(())
        })
    }

    pub(crate) fn with_exclusive_scene_transition<R>(
        &mut self,
        operation: &'static str,
        transition: impl FnOnce(&mut Self, &mut ExclusiveTransition<'_>) -> Result<R, String>,
    ) -> Result<R, String> {
        let context = self.context.clone();
        let mut exclusive = context
            .transactions()
            .begin_exclusive_transition(operation)
            .map_err(|error| error.to_string())?;
        self.cancel_gizmo_transaction()?;
        transition(self, &mut exclusive)
    }

    pub fn mark_project_open(&mut self) {
        self.session_mode = EditorSessionMode::Project;
        self.project_open = true;
    }

    pub fn set_session_mode(&mut self, session_mode: EditorSessionMode) {
        self.session_mode = session_mode;
    }

    pub fn set_welcome_snapshot(&mut self, welcome: WelcomePaneSnapshot) {
        self.welcome = welcome;
        if self.session_mode == EditorSessionMode::Welcome {
            self.status_line = self.welcome.status_message.clone();
        }
    }

    pub fn project_scene(&self) -> Option<Scene> {
        self.world.try_snapshot()
    }

    pub fn has_project_world(&self) -> bool {
        self.world.is_loaded()
    }
}
use std::path::Path;
