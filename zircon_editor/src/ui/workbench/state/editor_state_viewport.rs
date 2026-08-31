use zircon_runtime_interface::math::UVec2;

use crate::core::commands::CommandEvalCtx;
use crate::core::editing::authoring_world::AuthoringWorldAccessError;
use crate::core::editing::command::EditorCommand;
use crate::core::editing::interactive_transform::InteractiveTransformSession;
use crate::scene::viewport::{SceneViewportChromeSettings, SceneViewportSettings};
use crate::scene::viewport::{ViewportCameraSnapshot, ViewportInput};
use crate::scene::viewport::{ViewportFeedback, ViewportTransformRequest};
use crate::ui::binding::ViewportCommand;

use super::{
    editor_state::EditorState, EditorViewportStateError, GizmoTransactionError,
    GizmoTransactionPhase,
};

impl EditorState {
    pub(crate) fn viewport_camera_snapshot(
        &self,
    ) -> Result<Option<ViewportCameraSnapshot>, AuthoringWorldAccessError> {
        self.world
            .with_world(|scene| self.viewport_controller.current_camera(scene))
    }

    pub fn scene_viewport_settings(&self) -> SceneViewportChromeSettings {
        self.viewport_controller.chrome_settings()
    }

    pub fn update_scene_viewport_settings(
        &mut self,
        update: impl FnOnce(&mut SceneViewportSettings),
    ) -> bool {
        let mut next = self.viewport_controller.settings().clone();
        update(&mut next);
        if next == *self.viewport_controller.settings() {
            return false;
        }
        *self.viewport_controller.settings_mut() = next;
        true
    }

    pub(crate) fn project_command_eval_ctx(&self, context: CommandEvalCtx) -> CommandEvalCtx {
        self.viewport_controller.project_command_eval_ctx(context)
    }

    pub fn frame_selection(&mut self) -> Result<bool, EditorViewportStateError> {
        let Some(node_id) = self.viewport_controller.selection().active_primary() else {
            return Ok(false);
        };
        let Some(outcome) = self.world.with_world_mut(|scene| {
            self.viewport_controller
                .apply_command(Some(scene), &ViewportCommand::FrameSelection)
        })?
        else {
            return Ok(false);
        };
        let (feedback, post_callback_error) = outcome.into_parts();
        if let Some(error) = post_callback_error {
            return Err(error.into());
        }
        feedback.map_err(EditorViewportStateError::ViewportController)?;
        self.set_status_line(format!("Framed node {node_id}"));
        Ok(true)
    }

    pub fn handle_viewport_input(
        &mut self,
        input: ViewportInput,
    ) -> Result<ViewportFeedback, EditorViewportStateError> {
        let selected_before = self.viewport_controller.selection().active_primary();
        let was_handle_drag = self.viewport_controller.is_handle_drag_active();
        if was_handle_drag {
            if let Err(error) = self.transactions().ensure_mutation_available() {
                return Err(EditorViewportStateError::StateMutation(
                    self.rollback_interactive_transform(GizmoTransactionError::EditCommand {
                        phase: GizmoTransactionPhase::MutationPreflight,
                        source: error,
                    }),
                ));
            }
        }
        let outcome = match self
            .world
            .with_world_mut(|scene| self.viewport_controller.handle_input(scene, input))
        {
            Ok(Some(outcome)) => outcome,
            Ok(None) => return Ok(ViewportFeedback::default()),
            Err(error) if was_handle_drag => {
                return Err(EditorViewportStateError::StateMutation(
                    self.rollback_interactive_transform(error.into()),
                ));
            }
            Err(error) => return Err(error.into()),
        };
        let (feedback, post_callback_error) = outcome.into_parts();
        if let Some(error) = post_callback_error {
            return Err(EditorViewportStateError::StateMutation(
                self.rollback_interactive_transform(error.into()),
            ));
        }
        let mut feedback = feedback.map_err(EditorViewportStateError::PointerRoute)?;
        let is_handle_drag = self.viewport_controller.is_handle_drag_active();

        if !was_handle_drag && is_handle_drag {
            if let Err(error) = self.begin_interactive_transform() {
                return Err(EditorViewportStateError::StateMutation(
                    self.rollback_interactive_transform(error),
                ));
            }
        }

        if let Some(request) = feedback.transform_request.take() {
            let primary = request.primary;
            if let Err(error) = self.apply_interactive_transform_request(request) {
                return Err(EditorViewportStateError::StateMutation(
                    self.rollback_interactive_transform(error),
                ));
            }
            feedback.transformed_node = Some(primary);
        }

        if was_handle_drag && !is_handle_drag {
            self.finish_interactive_transform()
                .map_err(EditorViewportStateError::StateMutation)?;
        }

        let selected_after = self.viewport_controller.selection().active_primary();
        if feedback.transformed_node.is_some() || selected_before != selected_after {
            self.sync_selection_state();
        }
        if let Some(axis) = feedback.hovered_axis {
            self.set_status_line(format!("Hover gizmo axis {:?}", axis));
        }
        Ok(feedback)
    }

    fn apply_interactive_transform_request(
        &mut self,
        request: ViewportTransformRequest,
    ) -> Result<(), GizmoTransactionError> {
        let document = self.active_scene_document;
        let session = self
            .interactive_transform
            .as_mut()
            .ok_or(GizmoTransactionError::TransactionContextMissing)?;
        let viewport = &mut self.viewport_controller;
        let outcome = self
            .world
            .with_world_mut(|scene| {
                let active_camera = scene.active_camera();
                let active_camera_transform_before = scene.world_transform(active_camera);
                session.preview(scene, document, request.primary, request.target_pivot_world)?;
                viewport.resync_after_interactive_transform(
                    scene,
                    request.primary,
                    active_camera,
                    active_camera_transform_before,
                );
                Ok(())
            })?
            .ok_or(GizmoTransactionError::NoProjectOpen)?;
        let (result, post_callback_error) = outcome.into_parts();
        if let Some(error) = post_callback_error {
            return Err(error.into());
        }
        result
    }

    pub(crate) fn begin_interactive_transform(&mut self) -> Result<bool, GizmoTransactionError> {
        if self.interactive_transform.is_some() {
            return Ok(false);
        }
        self.transactions()
            .ensure_mutation_available()
            .map_err(|source| GizmoTransactionError::EditCommand {
                phase: GizmoTransactionPhase::MutationPreflight,
                source,
            })?;
        let Some(primary) = self.viewport_controller.selection().active_primary() else {
            return Ok(false);
        };
        let Some(spec) = self.viewport_controller.active_interactive_transform_spec() else {
            return Ok(false);
        };
        let pivot_mode = self.viewport_controller.interactive_transform_pivot_mode();
        let document = self
            .active_scene_document
            .ok_or(GizmoTransactionError::SceneDocumentNotActive)?;
        let selected = self
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let session = self
            .world
            .with_world(|scene| {
                InteractiveTransformSession::begin(
                    scene, &selected, primary, spec, pivot_mode, document,
                )
            })?
            .ok_or(GizmoTransactionError::NoProjectOpen)??;
        self.interactive_transform = Some(session);
        Ok(true)
    }

    pub(crate) fn finish_interactive_transform(&mut self) -> Result<bool, GizmoTransactionError> {
        let Some(session) = self.interactive_transform.as_ref() else {
            return Ok(false);
        };
        let document = self.active_scene_document;
        let label = session.spec().kind().history_label();
        let command = self
            .world
            .with_world(|scene| session.finish(scene, document))?
            .ok_or(GizmoTransactionError::NoProjectOpen)??;
        let Some(command) = command else {
            self.interactive_transform = None;
            return Ok(false);
        };
        if let Err(error) =
            self.execute_gizmo_scene_command(label, EditorCommand::applied_transform_batch(command))
        {
            return Err(self.rollback_interactive_transform(error));
        }
        self.interactive_transform = None;
        Ok(true)
    }

    pub(crate) fn prepare_non_gizmo_scene_action(&mut self) -> Result<(), GizmoTransactionError> {
        self.cancel_interactive_transform().map(|_| ())
    }

    pub(crate) fn cancel_interactive_transform(&mut self) -> Result<bool, GizmoTransactionError> {
        let session = self.interactive_transform.take();
        let had_transaction = session.is_some() || self.viewport_controller.is_handle_drag_active();
        if !had_transaction {
            return Ok(false);
        }
        let reset = self.reset_interactive_transform(session);
        self.sync_selection_state();
        reset?;
        Ok(true)
    }

    fn rollback_interactive_transform(
        &mut self,
        cause: GizmoTransactionError,
    ) -> GizmoTransactionError {
        let session = self.interactive_transform.take();
        let rollback = self.reset_interactive_transform(session);
        self.sync_selection_state();
        match rollback {
            Ok(()) => cause,
            Err(rollback) => GizmoTransactionError::RollbackFailed {
                cause: Box::new(cause),
                rollback: Box::new(rollback),
            },
        }
    }

    fn reset_interactive_transform(
        &mut self,
        session: Option<InteractiveTransformSession>,
    ) -> Result<(), GizmoTransactionError> {
        let document = self.active_scene_document;
        let viewport = &mut self.viewport_controller;
        match self.world.with_world_mut(|scene| {
            let restore_result = match session {
                Some(session) => session
                    .cancel(scene, document)
                    .map_err(GizmoTransactionError::from),
                None => Ok(()),
            };
            viewport.reset_from_scene(Some(scene));
            restore_result
        }) {
            Ok(Some(outcome)) => {
                let (result, post_callback_error) = outcome.into_parts();
                if let Some(error) = post_callback_error {
                    return Err(error.into());
                }
                result
            }
            Ok(None) => {
                viewport.reset_from_scene(None);
                Err(GizmoTransactionError::NoProjectOpen)
            }
            Err(error) => {
                viewport.reset_from_scene(None);
                Err(error.into())
            }
        }
    }

    pub fn apply_viewport_command(
        &mut self,
        command: &ViewportCommand,
    ) -> Result<ViewportFeedback, EditorViewportStateError> {
        match command {
            ViewportCommand::LeftPressed { .. } | ViewportCommand::LeftReleased
                if self.is_playing() =>
            {
                Ok(ViewportFeedback::default())
            }
            ViewportCommand::FrameSelection if self.is_playing() => Ok(ViewportFeedback::default()),
            ViewportCommand::PointerMoved { x, y } if self.is_playing() => self
                .handle_play_viewport_navigation(ViewportInput::PointerMoved(
                    zircon_runtime_interface::math::Vec2::new(*x, *y),
                )),
            ViewportCommand::RightPressed { x, y } if self.is_playing() => self
                .handle_play_viewport_navigation(ViewportInput::RightPressed(
                    zircon_runtime_interface::math::Vec2::new(*x, *y),
                )),
            ViewportCommand::RightReleased if self.is_playing() => {
                self.handle_play_viewport_navigation(ViewportInput::RightReleased)
            }
            ViewportCommand::MiddlePressed { x, y } if self.is_playing() => self
                .handle_play_viewport_navigation(ViewportInput::MiddlePressed(
                    zircon_runtime_interface::math::Vec2::new(*x, *y),
                )),
            ViewportCommand::MiddleReleased if self.is_playing() => {
                self.handle_play_viewport_navigation(ViewportInput::MiddleReleased)
            }
            ViewportCommand::Scrolled { delta } if self.is_playing() => {
                self.handle_play_viewport_navigation(ViewportInput::Scrolled(*delta))
            }
            ViewportCommand::Resized { width, height } if self.is_playing() => self
                .handle_play_viewport_navigation(ViewportInput::Resized(UVec2::new(
                    *width, *height,
                ))),
            ViewportCommand::SetGizmosEnabled(_) if self.is_playing() => {
                Ok(ViewportFeedback::default())
            }
            ViewportCommand::ActivateSceneMode(_) | ViewportCommand::SetPivotMode(_) => {
                self.cancel_interactive_transform()
                    .map_err(EditorViewportStateError::StateMutation)?;
                self.viewport_controller
                    .apply_command(None, command)
                    .map_err(EditorViewportStateError::ViewportController)
            }
            ViewportCommand::CancelInteraction => {
                let transformed_node = self
                    .interactive_transform
                    .as_ref()
                    .map(InteractiveTransformSession::primary_root)
                    .or_else(|| self.viewport_controller.selection().active_primary());
                let mut feedback = ViewportFeedback::default();
                if self
                    .cancel_interactive_transform()
                    .map_err(EditorViewportStateError::StateMutation)?
                {
                    feedback.transformed_node = transformed_node;
                }
                self.viewport_controller.cancel_interaction();
                Ok(feedback)
            }
            ViewportCommand::PointerMoved { x, y } => self.handle_viewport_input(
                ViewportInput::PointerMoved(zircon_runtime_interface::math::Vec2::new(*x, *y)),
            ),
            ViewportCommand::LeftPressed {
                x,
                y,
                selection_mutation,
            } => self.handle_viewport_input(ViewportInput::LeftPressed {
                position: zircon_runtime_interface::math::Vec2::new(*x, *y),
                selection_mutation: *selection_mutation,
            }),
            ViewportCommand::LeftReleased => {
                self.handle_viewport_input(ViewportInput::LeftReleased)
            }
            ViewportCommand::RightPressed { x, y } => self.handle_viewport_input(
                ViewportInput::RightPressed(zircon_runtime_interface::math::Vec2::new(*x, *y)),
            ),
            ViewportCommand::RightReleased => {
                self.handle_viewport_input(ViewportInput::RightReleased)
            }
            ViewportCommand::MiddlePressed { x, y } => self.handle_viewport_input(
                ViewportInput::MiddlePressed(zircon_runtime_interface::math::Vec2::new(*x, *y)),
            ),
            ViewportCommand::MiddleReleased => {
                self.handle_viewport_input(ViewportInput::MiddleReleased)
            }
            ViewportCommand::Scrolled { delta } => {
                self.handle_viewport_input(ViewportInput::Scrolled(*delta))
            }
            ViewportCommand::Resized { width, height } => {
                self.handle_viewport_input(ViewportInput::Resized(UVec2::new(*width, *height)))
            }
            ViewportCommand::FrameSelection => {
                let Some(outcome) = self.world.with_world_mut(|scene| {
                    self.viewport_controller.apply_command(Some(scene), command)
                })?
                else {
                    return Ok(ViewportFeedback::default());
                };
                let (feedback, post_callback_error) = outcome.into_parts();
                if let Some(error) = post_callback_error {
                    return Err(error.into());
                }
                feedback.map_err(EditorViewportStateError::ViewportController)
            }
            _ => self
                .viewport_controller
                .apply_command(None, command)
                .map_err(EditorViewportStateError::ViewportController),
        }
    }

    fn handle_play_viewport_navigation(
        &mut self,
        input: ViewportInput,
    ) -> Result<ViewportFeedback, EditorViewportStateError> {
        let Some(outcome) = self.world.with_world_mut(|scene| {
            self.viewport_controller
                .handle_editor_camera_input(scene, input)
        })?
        else {
            return Ok(ViewportFeedback::default());
        };
        let (feedback, post_callback_error) = outcome.into_parts();
        if let Some(error) = post_callback_error {
            return Err(error.into());
        }
        feedback.map_err(EditorViewportStateError::PointerRoute)
    }
}

#[cfg(test)]
mod play_viewport_route_contract_tests {
    #[test]
    fn play_pointer_commands_use_the_navigation_only_entry_and_block_authoring_frame_selection() {
        let source = include_str!("editor_state_viewport.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(production, _)| production);

        assert!(production.contains("handle_play_viewport_navigation"));
        assert!(production.contains("ViewportCommand::FrameSelection if self.is_playing()"));
        assert!(production.contains("ViewportCommand::LeftPressed { .. }"));
        assert!(production.contains("ViewportCommand::LeftReleased"));
    }
}
