use crate::{ViewportFeedback, ViewportInput};
use crate::ui::ViewportCommand;
use zircon_math::UVec2;
use zircon_scene::SceneViewportSettings;

use super::editor_state::EditorState;

impl EditorState {
    pub fn scene_viewport_settings(&self) -> &SceneViewportSettings {
        self.viewport_controller.settings()
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

    pub fn frame_selection(&mut self) -> bool {
        let Some(node_id) = self.viewport_controller.selected_node() else {
            return false;
        };
        let _ = self.world.try_with_world_mut(|scene| {
            self.viewport_controller
                .apply_command(Some(scene), &ViewportCommand::FrameSelection)
        });
        self.status_line = format!("Framed node {node_id}");
        true
    }

    pub fn handle_viewport_input(&mut self, input: ViewportInput) -> ViewportFeedback {
        let selected_before = self.viewport_controller.selected_node();
        let was_handle_drag = self.viewport_controller.is_handle_drag_active();
        let Some(feedback) = self
            .world
            .try_with_world_mut(|scene| self.viewport_controller.handle_input(scene, input))
        else {
            return ViewportFeedback::default();
        };
        let is_handle_drag = self.viewport_controller.is_handle_drag_active();

        if !was_handle_drag && is_handle_drag {
            let selected = self.viewport_controller.selected_node();
            let history = &mut self.history;
            let _ = self
                .world
                .try_with_world(|scene| history.begin_drag(scene, selected));
        }

        if was_handle_drag && !is_handle_drag {
            let selected = self.viewport_controller.selected_node();
            let history = &mut self.history;
            let command = self
                .world
                .try_with_world(|scene| history.end_drag(scene, selected))
                .and_then(Result::ok)
                .flatten();
            if let Some(command) = command {
                self.history.push(command);
            }
        }

        let selected_after = self.viewport_controller.selected_node();

        if feedback.transformed_node.is_some() || selected_before != selected_after {
            self.sync_selection_state();
        }

        if let Some(axis) = feedback.hovered_axis {
            self.status_line = format!("Hover gizmo axis {:?}", axis);
        }
        feedback
    }

    pub fn apply_viewport_command(&mut self, command: &ViewportCommand) -> ViewportFeedback {
        match command {
            ViewportCommand::PointerMoved { x, y } => self
                .handle_viewport_input(ViewportInput::PointerMoved(zircon_math::Vec2::new(*x, *y))),
            ViewportCommand::LeftPressed { x, y } => self
                .handle_viewport_input(ViewportInput::LeftPressed(zircon_math::Vec2::new(*x, *y))),
            ViewportCommand::LeftReleased => {
                self.handle_viewport_input(ViewportInput::LeftReleased)
            }
            ViewportCommand::RightPressed { x, y } => self
                .handle_viewport_input(ViewportInput::RightPressed(zircon_math::Vec2::new(*x, *y))),
            ViewportCommand::RightReleased => {
                self.handle_viewport_input(ViewportInput::RightReleased)
            }
            ViewportCommand::MiddlePressed { x, y } => self.handle_viewport_input(
                ViewportInput::MiddlePressed(zircon_math::Vec2::new(*x, *y)),
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
                .unwrap_or_default(),
            _ => self.viewport_controller.apply_command(None, command),
        }
    }
}
