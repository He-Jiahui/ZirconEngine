mod world_space;

use std::time::Instant;

use super::super::{callback_dispatch, RetainedEditorHost};
use super::pointer_mapping::map_viewport_pointer_event;
use crate::scene::selection::SelectionMutation;
use crate::ui::host::PlayGizmoPointerOutcome;
use crate::ui::retained_host::{FrameRect, PaneSurfaceHostContext, UiHostWindow};
use world_space::world_space_ui_pointer_status;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
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
        let source_ui = self.callback_source_ui();
        let surface_key = source_ui
            .global::<PaneSurfaceHostContext>()
            .scene_viewport_surface_key();
        if let Some(surface_key) = surface_key.as_deref() {
            if !self.route_scene_viewport_surface(surface_key, event.kind) {
                return;
            }
            if let Some(size) = scene_viewport_surface_size(&source_ui, surface_key) {
                self.viewport_pointer_bridge
                    .update_viewport_frame(UiFrame::new(0.0, 0.0, size.width, size.height));
            }
        }
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

        let play_frame = source_ui
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

    pub(in crate::ui::retained_host::app) fn focus_scene_viewport_surface(
        &self,
        surface_key: &str,
    ) -> bool {
        let source_ui = self.callback_source_ui();
        match scene_viewport_surface_target(&source_ui, surface_key) {
            Some(SceneViewportSurfaceTarget::Scene(view_id)) => {
                if !self
                    .runtime
                    .route_scene_viewport_pointer(view_id, UiPointerEventKind::Down)
                {
                    return false;
                }
                source_ui
                    .global::<PaneSurfaceHostContext>()
                    .set_scene_viewport_surface_key(surface_key);
                true
            }
            Some(SceneViewportSurfaceTarget::OtherPane) => true,
            None => false,
        }
    }

    fn route_scene_viewport_surface(
        &self,
        surface_key: &str,
        event_kind: UiPointerEventKind,
    ) -> bool {
        let source_ui = self.callback_source_ui();
        let Some(SceneViewportSurfaceTarget::Scene(view_id)) =
            scene_viewport_surface_target(&source_ui, surface_key)
        else {
            return false;
        };
        self.runtime
            .route_scene_viewport_pointer(view_id, event_kind)
    }

    pub(super) fn callback_source_ui(&self) -> UiHostWindow {
        self.callback_source_window
            .as_ref()
            .and_then(|window_id| self.native_window_presenters.window(window_id))
            .unwrap_or_else(|| self.ui.clone_strong())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SceneViewportSurfaceTarget {
    Scene(crate::core::editor_event::ViewInstanceId),
    OtherPane,
}

pub(super) fn scene_viewport_surface_target(
    ui: &UiHostWindow,
    surface_key: &str,
) -> Option<SceneViewportSurfaceTarget> {
    let generation = ui.get_host_presentation_generation();
    let presentation = generation.structure();
    let scene = &presentation.host_scene_data;
    let pane = scene
        .document_surfaces()
        .iter()
        .find(|surface| surface.surface_key.as_str() == surface_key)
        .map(|surface| &surface.pane)
        .or_else(|| {
            [
                (&scene.left_dock.surface_key, &scene.left_dock.pane),
                (&scene.right_dock.surface_key, &scene.right_dock.pane),
                (&scene.bottom_dock.surface_key, &scene.bottom_dock.pane),
            ]
            .into_iter()
            .find(|(key, _)| key.as_str() == surface_key)
            .map(|(_, pane)| pane)
        })
        .or_else(|| {
            scene
                .floating_layer
                .floating_windows
                .iter()
                .find(|window| window.window_id.as_str() == surface_key)
                .map(|window| &window.active_pane)
        })
        .or_else(|| {
            presentation
                .native_floating_surface_data
                .floating_windows
                .iter()
                .find(|window| window.window_id.as_str() == surface_key)
                .map(|window| &window.active_pane)
        });
    let pane = pane?;
    Some(if pane.kind.as_str() == "Scene" {
        SceneViewportSurfaceTarget::Scene(crate::core::editor_event::ViewInstanceId::new(
            pane.id.as_str(),
        ))
    } else {
        SceneViewportSurfaceTarget::OtherPane
    })
}

/// Resolves the committed Scene leaf for a toolbar surface without changing focus or creating a
/// viewport session. Toolbar commands must use this committed identity all the way through the
/// binding journal and executor; pointer-down routing owns the separate focus path above.
pub(super) fn committed_scene_viewport_id_for_surface(
    ui: &UiHostWindow,
    surface_key: &str,
) -> Option<crate::core::editor_event::ViewInstanceId> {
    match scene_viewport_surface_target(ui, surface_key) {
        Some(SceneViewportSurfaceTarget::Scene(view_id)) => Some(view_id),
        Some(SceneViewportSurfaceTarget::OtherPane) | None => None,
    }
}

pub(in crate::ui::retained_host::app) fn scene_viewport_surface_size(
    ui: &UiHostWindow,
    surface_key: &str,
) -> Option<UiSize> {
    let generation = ui.get_host_presentation_generation();
    let presentation = generation.structure();
    let scene = &presentation.host_scene_data;
    let size = scene
        .document_surfaces()
        .iter()
        .find(|surface| surface.surface_key.as_str() == surface_key)
        .map(|surface| (&surface.content_frame.width, &surface.content_frame.height))
        .or_else(|| {
            [
                (&scene.left_dock.surface_key, &scene.left_dock.content_frame),
                (
                    &scene.right_dock.surface_key,
                    &scene.right_dock.content_frame,
                ),
                (
                    &scene.bottom_dock.surface_key,
                    &scene.bottom_dock.content_frame,
                ),
            ]
            .into_iter()
            .find(|(key, _)| key.as_str() == surface_key)
            .map(|(_, frame)| (&frame.width, &frame.height))
        })
        .map(|(width, height)| UiSize::new((*width).max(0.0), (*height).max(0.0)))
        .or_else(|| {
            scene
                .floating_layer
                .floating_windows
                .iter()
                .find(|window| window.window_id.as_str() == surface_key)
                .map(|window| {
                    UiSize::new(
                        window.frame.width.max(0.0),
                        (window.frame.height - scene.floating_layer.header_height_px).max(0.0),
                    )
                })
        })
        .or_else(|| {
            presentation
                .native_floating_surface_data
                .floating_windows
                .iter()
                .find(|window| window.window_id.as_str() == surface_key)
                .map(|window| {
                    UiSize::new(
                        window.frame.width.max(0.0),
                        (window.frame.height
                            - presentation.native_floating_surface_data.header_height_px)
                            .max(0.0),
                    )
                })
        })?;
    (size.width > 0.0 && size.height > 0.0).then_some(size)
}

pub(in crate::ui::retained_host::app) fn scene_viewport_surface_damage_frame(
    ui: &UiHostWindow,
    surface_key: &str,
) -> FrameRect {
    let generation = ui.get_host_presentation_generation();
    let presentation = generation.structure();
    if presentation.host_shell.native_floating_window_mode {
        return ui.get_host_window_bootstrap().shell_frame;
    }
    let scene = &presentation.host_scene_data;
    scene
        .document_surfaces()
        .iter()
        .find(|surface| surface.surface_key.as_str() == surface_key)
        .map(|surface| surface.region_frame.clone())
        .or_else(|| {
            [
                (&scene.left_dock.surface_key, &scene.left_dock.region_frame),
                (
                    &scene.right_dock.surface_key,
                    &scene.right_dock.region_frame,
                ),
                (
                    &scene.bottom_dock.surface_key,
                    &scene.bottom_dock.region_frame,
                ),
            ]
            .into_iter()
            .find(|(key, _)| key.as_str() == surface_key)
            .map(|(_, frame)| frame.clone())
        })
        .or_else(|| {
            scene
                .floating_layer
                .floating_windows
                .iter()
                .find(|window| window.window_id.as_str() == surface_key)
                .map(|window| window.frame.clone())
        })
        .unwrap_or_else(|| ui.get_host_window_bootstrap().shell_frame)
}

#[cfg(test)]
#[path = "tests/pointer_event.rs"]
mod tests;
