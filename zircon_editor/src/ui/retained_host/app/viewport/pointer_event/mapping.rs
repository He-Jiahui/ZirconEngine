use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
};

const VIEWPORT_POINTER_DOWN: i32 = 0;
const VIEWPORT_POINTER_MOVE: i32 = 1;
const VIEWPORT_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_SCROLL: i32 = 3;
const VIEWPORT_POINTER_CANCEL: i32 = 4;

const VIEWPORT_POINTER_BUTTON_NONE: i32 = 0;
const VIEWPORT_POINTER_BUTTON_PRIMARY: i32 = 1;
const VIEWPORT_POINTER_BUTTON_SECONDARY: i32 = 2;
const VIEWPORT_POINTER_BUTTON_MIDDLE: i32 = 3;

pub(super) fn map_viewport_pointer_event(
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
        VIEWPORT_POINTER_CANCEL => UiPointerEventKind::Cancel,
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
