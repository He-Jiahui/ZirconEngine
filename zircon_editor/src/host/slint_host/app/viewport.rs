use super::*;
use zircon_ui::{UiPointerButton, UiPointerEvent, UiPointerEventKind};

const VIEWPORT_POINTER_DOWN: i32 = 0;
const VIEWPORT_POINTER_MOVE: i32 = 1;
const VIEWPORT_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_SCROLL: i32 = 3;

const VIEWPORT_POINTER_BUTTON_NONE: i32 = 0;
const VIEWPORT_POINTER_BUTTON_PRIMARY: i32 = 1;
const VIEWPORT_POINTER_BUTTON_SECONDARY: i32 = 2;
const VIEWPORT_POINTER_BUTTON_MIDDLE: i32 = 3;

impl SlintEditorHost {
    pub(super) fn viewport_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        delta: f32,
    ) {
        self.recompute_if_dirty();
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

        match callback_dispatch::dispatch_viewport_pointer_event(
            &self.runtime,
            &mut self.viewport_pointer_bridge,
            event,
        ) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn viewport_toolbar_pointer_clicked(
        &mut self,
        surface_key: &str,
        control_id: &str,
        control_x: f32,
        control_y: f32,
        control_width: f32,
        control_height: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.recompute_if_dirty();
        self.focus_callback_source_window();
        self.viewport_toolbar_pointer_bridge
            .sync(build_viewport_toolbar_pointer_layout([surface_key]));
        match callback_dispatch::dispatch_shared_viewport_toolbar_pointer_click(
            &self.runtime,
            &self.viewport_toolbar_bridge,
            &mut self.viewport_toolbar_pointer_bridge,
            surface_key,
            control_id,
            control_x,
            control_y,
            control_width,
            control_height,
            UiPoint::new(point_x, point_y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}

fn map_viewport_pointer_event(
    kind: i32,
    button: i32,
    x: f32,
    y: f32,
    delta: f32,
) -> Result<UiPointerEvent, String> {
    let kind = match kind {
        VIEWPORT_POINTER_DOWN => UiPointerEventKind::Down,
        VIEWPORT_POINTER_MOVE => UiPointerEventKind::Move,
        VIEWPORT_POINTER_UP => UiPointerEventKind::Up,
        VIEWPORT_POINTER_SCROLL => UiPointerEventKind::Scroll,
        _ => return Err(format!("unknown viewport pointer kind {kind}")),
    };

    let mut event = UiPointerEvent::new(kind, UiPoint::new(x, y));
    if let Some(button) = map_viewport_pointer_button(button)? {
        event = event.with_button(button);
    }
    if kind == UiPointerEventKind::Scroll {
        event = event.with_scroll_delta(delta);
    }
    Ok(event)
}

fn map_viewport_pointer_button(button: i32) -> Result<Option<UiPointerButton>, String> {
    match button {
        VIEWPORT_POINTER_BUTTON_NONE => Ok(None),
        VIEWPORT_POINTER_BUTTON_PRIMARY => Ok(Some(UiPointerButton::Primary)),
        VIEWPORT_POINTER_BUTTON_SECONDARY => Ok(Some(UiPointerButton::Secondary)),
        VIEWPORT_POINTER_BUTTON_MIDDLE => Ok(Some(UiPointerButton::Middle)),
        _ => Err(format!("unknown viewport pointer button {button}")),
    }
}
