use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::{Transform, UVec2};

use crate::core::commands::CommandEvalCtx;
use crate::core::editing::command::{EditorCommand, NodeEditState};
use crate::core::editing::engine::EditCommandError;
use crate::scene::viewport::ViewportInput;
use crate::scene::viewport::{SceneViewportChromeSettings, SceneViewportSettings};
use crate::scene::viewport::{ViewportFeedback, ViewportTransformPreview};
use crate::ui::binding::ViewportCommand;

use super::editor_state::EditorState;

#[derive(Clone, Debug)]
pub(in crate::ui::workbench) struct GizmoTransactionCapture {
    node_id: NodeId,
    initial: Transform,
    latest: Transform,
}

impl EditorState {
    pub fn scene_viewport_settings(&self) -> SceneViewportChromeSettings {
        self.viewport_controller.chrome_settings()
    }

    pub fn update_scene_viewport_settings(
        &mut self,
        update: impl FnOnce(&mut SceneViewportSettings),
    ) -> bool {
        let mut next = self.viewport_controller.settings().clone();
        update(&mut next);
        if self.is_playing() {
            next.gizmos_enabled = false;
        }
        if next == *self.viewport_controller.settings() {
            return false;
        }
        *self.viewport_controller.settings_mut() = next;
        true
    }

    pub(crate) fn project_command_eval_ctx(&self, context: CommandEvalCtx) -> CommandEvalCtx {
        self.viewport_controller.project_command_eval_ctx(context)
    }

    pub fn frame_selection(&mut self) -> bool {
        let Some(node_id) = self.viewport_controller.selection().active_primary() else {
            return false;
        };
        let _ = self.world.try_with_world_mut(|scene| {
            self.viewport_controller
                .apply_command(Some(scene), &ViewportCommand::FrameSelection)
        });
        self.set_status_line(format!("Framed node {node_id}"));
        true
    }

    pub fn handle_viewport_input(
        &mut self,
        input: ViewportInput,
    ) -> Result<ViewportFeedback, String> {
        let selected_before = self.viewport_controller.selection().active_primary();
        let was_handle_drag = self.viewport_controller.is_handle_drag_active();
        if was_handle_drag {
            if let Err(error) = self.transactions().ensure_mutation_available() {
                return Err(self.rollback_gizmo_transaction(error.to_string()));
            }
        }
        let Some(mut feedback) = self
            .world
            .try_with_world_mut(|scene| self.viewport_controller.handle_input(scene, input))
        else {
            return Ok(ViewportFeedback::default());
        };
        let is_handle_drag = self.viewport_controller.is_handle_drag_active();

        if !was_handle_drag && is_handle_drag {
            if let Err(error) = self.begin_gizmo_transaction() {
                return Err(self.rollback_gizmo_transaction(error));
            }
        }

        if let Some(preview) = feedback.transform_preview.take() {
            let node_id = preview.node_id;
            if let Err(error) = self.apply_gizmo_transform_preview(preview) {
                return Err(self.rollback_gizmo_transaction(error));
            }
            feedback.transformed_node = Some(node_id);
            self.record_gizmo_transaction_step()?;
        }

        if was_handle_drag && !is_handle_drag {
            self.finish_gizmo_transaction()?;
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

    fn apply_gizmo_transform_preview(
        &mut self,
        preview: ViewportTransformPreview,
    ) -> Result<(), String> {
        let viewport = &mut self.viewport_controller;
        self.world
            .try_with_world_mut(|scene| {
                scene
                    .update_transform(preview.node_id, preview.transform)
                    .map_err(|error| error.to_string())?;
                viewport.accept_transform_preview(scene, preview);
                Ok(())
            })
            .ok_or_else(|| "No project open".to_string())?
    }

    pub(crate) fn begin_gizmo_transaction(&mut self) -> Result<bool, String> {
        if self.gizmo_transaction.is_some() {
            return Ok(false);
        }
        self.transactions()
            .ensure_mutation_available()
            .map_err(|error| error.to_string())?;
        let Some(node_id) = self.viewport_controller.selection().active_primary() else {
            return Ok(false);
        };
        let initial = self.capture_node_transform(node_id)?;
        self.gizmo_transaction = Some(GizmoTransactionCapture {
            node_id,
            initial,
            latest: initial,
        });
        Ok(true)
    }

    pub(crate) fn record_gizmo_transaction_step(&mut self) -> Result<bool, String> {
        let Some(capture) = self.gizmo_transaction.as_ref() else {
            return Ok(false);
        };
        let node_id = capture.node_id;
        let before = capture.latest;
        let after = match self.capture_node_transform(node_id) {
            Ok(after) => after,
            Err(error) => return Err(self.rollback_gizmo_transaction(error)),
        };
        if before == after {
            return Ok(false);
        }
        if let Some(capture) = self.gizmo_transaction.as_mut() {
            capture.latest = after;
        }
        Ok(true)
    }

    pub(crate) fn finish_gizmo_transaction(&mut self) -> Result<bool, String> {
        self.record_gizmo_transaction_step()?;
        let Some(capture) = self.gizmo_transaction.as_ref() else {
            return Ok(false);
        };
        if capture.initial == capture.latest {
            self.gizmo_transaction = None;
            return Ok(false);
        }
        let node_id = capture.node_id;
        let initial = capture.initial;
        let after = match self.capture_scene_command(|scene| NodeEditState::capture(scene, node_id))
        {
            Ok(after) => after,
            Err(error) => return Err(self.rollback_gizmo_transaction(error)),
        };
        let mut before = after.clone();
        before.transform = initial;
        let Some(command) = EditorCommand::applied_transform(node_id, before, after) else {
            self.gizmo_transaction = None;
            return Ok(false);
        };
        if let Err(error) = self.execute_gizmo_scene_command("Move scene node", command) {
            return Err(self.rollback_gizmo_transaction(error));
        }
        self.gizmo_transaction = None;
        Ok(true)
    }

    pub(crate) fn prepare_non_gizmo_scene_action(&mut self) -> Result<(), String> {
        self.cancel_gizmo_transaction().map(|_| ())
    }

    pub(crate) fn cancel_gizmo_transaction(&mut self) -> Result<bool, String> {
        let restore = self
            .gizmo_transaction
            .as_ref()
            .map(|capture| (capture.node_id, capture.initial));
        let had_transaction = restore.is_some() || self.viewport_controller.is_handle_drag_active();
        if !had_transaction {
            return Ok(false);
        }
        let reset = self.reset_gizmo_interaction(restore);
        self.gizmo_transaction = None;
        self.sync_selection_state();
        reset?;
        Ok(true)
    }

    fn rollback_gizmo_transaction(&mut self, cause: String) -> String {
        let restore = self
            .gizmo_transaction
            .as_ref()
            .map(|capture| (capture.node_id, capture.initial));
        let rollback = self.reset_gizmo_interaction(restore);
        self.gizmo_transaction = None;
        self.sync_selection_state();
        match rollback {
            Ok(()) => cause,
            Err(error) => format!("{cause}; gizmo transaction rollback failed: {error}"),
        }
    }

    fn reset_gizmo_interaction(
        &mut self,
        restore: Option<(NodeId, Transform)>,
    ) -> Result<(), String> {
        let viewport = &mut self.viewport_controller;
        match self.world.try_with_world_mut(|scene| {
            let restore_result = match restore {
                Some((node_id, transform)) => scene
                    .update_transform(node_id, transform)
                    .map(|_| ())
                    .map_err(|error| error.to_string()),
                None => Ok(()),
            };
            viewport.reset_from_scene(Some(scene));
            restore_result
        }) {
            Some(result) => result,
            None => {
                viewport.reset_from_scene(None);
                Err("No project open".to_string())
            }
        }
    }

    fn capture_node_transform(&self, node_id: NodeId) -> Result<Transform, String> {
        self.capture_scene_command(|scene| {
            scene
                .find_node(node_id)
                .map(|node| node.transform)
                .ok_or_else(|| EditCommandError::TargetMissing {
                    target: format!("scene node {node_id}"),
                })
        })
    }

    pub fn apply_viewport_command(
        &mut self,
        command: &ViewportCommand,
    ) -> Result<ViewportFeedback, String> {
        match command {
            ViewportCommand::SetGizmosEnabled(_) if self.is_playing() => {
                Ok(ViewportFeedback::default())
            }
            ViewportCommand::ActivateSceneMode(_) => {
                self.cancel_gizmo_transaction()?;
                self.viewport_controller.apply_command(None, command)
            }
            ViewportCommand::CancelInteraction => {
                let transformed_node = self
                    .gizmo_transaction
                    .as_ref()
                    .map(|capture| capture.node_id)
                    .or_else(|| self.viewport_controller.selection().active_primary());
                let mut feedback = ViewportFeedback::default();
                if self.cancel_gizmo_transaction()? {
                    feedback.transformed_node = transformed_node;
                }
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
            ViewportCommand::FrameSelection => self
                .world
                .try_with_world_mut(|scene| {
                    self.viewport_controller.apply_command(Some(scene), command)
                })
                .unwrap_or_else(|| Ok(ViewportFeedback::default())),
            _ => self.viewport_controller.apply_command(None, command),
        }
    }
}
