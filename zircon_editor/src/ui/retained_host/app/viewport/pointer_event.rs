mod mapping;
mod world_space;

use super::super::{callback_dispatch, RetainedEditorHost};
use mapping::map_viewport_pointer_event;
use world_space::world_space_ui_pointer_status;
use zircon_runtime_interface::ui::surface::UiPointerEventKind;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn viewport_pointer_event(
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
