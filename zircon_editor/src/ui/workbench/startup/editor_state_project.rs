use std::path::Path;

use zircon_runtime::scene::Scene;

use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditCommandError, ExclusiveTransition};
use crate::ui::workbench::state::{EditorState, EditorStateOperationError};

use super::{EditorSessionMode, WelcomePaneSnapshot};

impl EditorState {
    /// Refuses a route-driven world replacement while the current scene history is dirty.
    ///
    /// This is the host-side admission half of the scene transition protocol. Save/discard UI
    /// resolves the dirty state before retrying the same route; replacement itself never clears
    /// a dirty history as an implicit discard.
    pub(crate) fn prepare_scene_transition(&self) -> Result<(), EditorStateOperationError> {
        if let Some(history_context) = self.active_scene_history_context() {
            if self
                .transactions()
                .is_dirty(history_context)
                .map_err(EditorStateOperationError::from)?
            {
                return Err(EditorStateOperationError::SceneTransitionDirty);
            }
        }
        Ok(())
    }

    pub fn replace_world(
        &mut self,
        world: impl Into<AuthoringWorldSeed>,
        project_path: impl Into<String>,
    ) -> Result<(), EditorStateOperationError> {
        let project_path = project_path.into();
        self.with_exclusive_scene_transition("replace editor world", move |state, transition| {
            if let Some(history_context) = state.active_scene_history_context() {
                transition.clear_history_and_context::<CoreEditContext>(
                    history_context,
                    "CoreEditContext",
                    |context| {
                        context.clear_scene()?;
                        state.world.replace(world).map_err(|error| {
                            EditCommandError::ExternalEffect {
                                source: Box::new(error),
                            }
                        })
                    },
                )?;
            } else {
                state
                    .world
                    .replace(world)
                    .map_err(|error| EditCommandError::ExternalEffect {
                        source: Box::new(error),
                    })?;
            }
            state.clear_scene_document_binding();
            state.project_path = project_path;
            state
                .context
                .settings_mutations()
                .bind_project(Path::new(&state.project_path))?;
            state.session_mode = EditorSessionMode::Project;
            state.project_open = true;
            state.welcome = WelcomePaneSnapshot::default();
            *state.viewport_controller.selection_mut() = Default::default();
            state
                .world
                .with_world(|scene| state.viewport_controller.reset_from_scene(Some(scene)))?;
            state.sync_selection_state();
            Ok(())
        })
    }

    /// Replaces the clean active scene while preserving its project and document identity.
    pub(crate) fn reload_active_scene_world(
        &mut self,
        world: impl Into<AuthoringWorldSeed>,
    ) -> Result<(), EditorStateOperationError> {
        let history_context = self.scene_history_context()?;
        self.with_exclusive_scene_transition(
            "reload active editor scene",
            move |state, transition| {
                transition.clear_history_and_context::<CoreEditContext>(
                    history_context,
                    "CoreEditContext",
                    |context| {
                        context.clear_scene()?;
                        state.world.replace(world).map_err(|error| {
                            EditCommandError::ExternalEffect {
                                source: Box::new(error),
                            }
                        })
                    },
                )?;
                *state.viewport_controller.selection_mut() = Default::default();
                state
                    .world
                    .with_world(|scene| state.viewport_controller.reset_from_scene(Some(scene)))?;
                state.sync_selection_state();
                Ok(())
            },
        )
    }

    pub fn clear_project(
        &mut self,
        welcome: WelcomePaneSnapshot,
    ) -> Result<(), EditorStateOperationError> {
        self.with_exclusive_scene_transition("clear editor project", move |state, transition| {
            if let Some(history_context) = state.active_scene_history_context() {
                transition.clear_history_and_context::<CoreEditContext>(
                    history_context,
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
                )?;
            } else {
                state
                    .world
                    .clear()
                    .map_err(|error| EditCommandError::ExternalEffect {
                        source: Box::new(error),
                    })?;
            }
            state.clear_scene_document_binding();
            state.project_path.clear();
            state.context.settings_mutations().clear_project()?;
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
        transition: impl FnOnce(
            &mut Self,
            &mut ExclusiveTransition<'_>,
        ) -> Result<R, EditorStateOperationError>,
    ) -> Result<R, EditorStateOperationError> {
        let context = self.context.clone();
        let mut exclusive = context
            .transactions()
            .begin_exclusive_transition(operation)?;
        self.cancel_interactive_transform()?;
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
            self.set_status_line(self.welcome.status_message.clone());
        }
    }

    pub fn project_scene(
        &self,
    ) -> Result<Option<Scene>, crate::core::editing::authoring_world::AuthoringWorldAccessError>
    {
        self.world.snapshot()
    }

    pub fn has_project_world(&self) -> bool {
        self.world.is_loaded()
    }
}
