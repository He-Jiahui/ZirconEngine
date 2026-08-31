mod world_space;

use std::time::Instant;

use super::super::{callback_dispatch, RetainedEditorHost};
use super::pointer_mapping::map_viewport_pointer_event;
use crate::scene::selection::SelectionMutation;
use crate::ui::host::PlayGizmoPointerOutcome;
use crate::ui::retained_host::PaneSurfaceHostContext;
use world_space::world_space_ui_pointer_status;
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn scene_viewport_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        delta: f32,
        shift: bool,
        control: bool,
    ) {
        self.use_committed_pointer_layout();
        let event = match map_viewport_pointer_event(kind, button, x, y, delta) {
            Ok(event) => event,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        if event.kind != UiPointerEventKind::Move {
            self.focus_callback_source_window();
        }
        if event.kind == UiPointerEventKind::Cancel {
            if let Err(error) = self.play_viewport_pick.cancel() {
                self.set_status_line(error.to_string());
                self.ui.set_lifecycle_frame_update(Some(
                    super::super::PlayViewportPickConsumer::next_error_retry_deadline(),
                ));
            }
        }

        if let Some(route) = self.viewport.route_world_space_ui_pointer_event(
            event.kind,
            event.point.x,
            event.point.y,
        ) {
            if let Some(status) = world_space_ui_pointer_status(event.kind, &route.control_id) {
                self.set_status_line(status);
            }
            return;
        }

        let play_frame = self
            .ui
            .global::<PaneSurfaceHostContext>()
            .simulate_viewport_frame_identity();
        let play_gizmo: PlayGizmoPointerOutcome = match self.runtime.route_play_gizmo_pointer(
            play_frame.as_ref(),
            event.kind,
            event.button,
            event.point,
        ) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        if play_gizmo.supersedes_scene_pick() {
            if let Err(error) = self.play_viewport_pick.cancel() {
                self.set_status_line(error.to_string());
                self.ui.set_lifecycle_frame_update(Some(
                    super::super::PlayViewportPickConsumer::next_error_retry_deadline(),
                ));
            }
        }
        if let Some(status) = play_gizmo.status_line() {
            self.set_status_line(status);
        }
        if play_gizmo.presentation_changed() {
            self.mark_presentation_dirty();
        }
        if play_gizmo.consumed() {
            return;
        }

        if event.kind == UiPointerEventKind::Down && event.button == Some(UiPointerButton::Primary)
        {
            if let Some(frame) = play_frame {
                let mutation = SelectionMutation::from_modifier_flags(shift, control);
                match self
                    .play_viewport_pick
                    .request(&self.runtime, &frame, event.point, mutation)
                {
                    Ok(true) => self.ui.set_lifecycle_frame_update(Some(Instant::now())),
                    Ok(false) => {}
                    Err(error) => {
                        self.set_status_line(error.to_string());
                        self.ui.set_lifecycle_frame_update(Some(
                            super::super::PlayViewportPickConsumer::next_error_retry_deadline(),
                        ));
                    }
                }
            }
        }

        match callback_dispatch::dispatch_viewport_pointer_event(
            &self.runtime,
            &mut self.viewport_pointer_bridge,
            event,
            zircon_runtime_interface::ui::dispatch::UiInputModifiers {
                shift,
                control,
                ..Default::default()
            },
        ) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }
}
