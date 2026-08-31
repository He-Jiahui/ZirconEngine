use winit::{
    dpi::PhysicalPosition,
    event::{
        ButtonSource, ElementState, FingerId, MouseButton, MouseScrollDelta, PointerKind,
        PointerSource,
    },
};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerId,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
    window::{
        UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpEvent, UiWindowPlatformInputEvent,
    },
};

use super::window::{input_event, window_event};

const PIXEL_SCROLL_LINE_DELTA_SCALE: f32 = 0.1;

pub(super) fn translate_pointer_moved(
    context: UiWindowInputContext,
    position: PhysicalPosition<f64>,
    source: &PointerSource,
) -> Option<UiWindowInputPumpEvent> {
    let point = point_from_physical_position(position);
    match source {
        PointerSource::Touch { finger_id, .. } => Some(input_event(
            UiWindowPlatformInputEvent::touch_moved(context, pointer_id(*finger_id), point),
        )),
        PointerSource::Mouse | PointerSource::Unknown | PointerSource::TabletTool { .. } => {
            Some(window_event(
                &context,
                UiWindowEventKind::CursorMoved {
                    position: point,
                    delta: None,
                },
            ))
        }
    }
}

pub(super) fn translate_pointer_entered(
    context: &UiWindowInputContext,
    _position: PhysicalPosition<f64>,
    kind: &PointerKind,
) -> Option<UiWindowInputPumpEvent> {
    match kind {
        PointerKind::Mouse | PointerKind::Unknown | PointerKind::TabletTool(_) => {
            Some(window_event(context, UiWindowEventKind::CursorEntered))
        }
        PointerKind::Touch(_) => None,
    }
}

pub(super) fn translate_pointer_left(
    context: UiWindowInputContext,
    position: Option<PhysicalPosition<f64>>,
    kind: &PointerKind,
) -> Option<UiWindowInputPumpEvent> {
    let point = position
        .map(point_from_physical_position)
        .unwrap_or_default();
    match kind {
        PointerKind::Touch(finger_id) => Some(input_event(
            UiWindowPlatformInputEvent::touch_canceled(context, pointer_id(*finger_id), point),
        )),
        PointerKind::Mouse | PointerKind::Unknown | PointerKind::TabletTool(_) => {
            Some(window_event(&context, UiWindowEventKind::CursorLeft))
        }
    }
}

pub(super) fn translate_mouse_wheel_event(
    context: UiWindowInputContext,
    point: UiPoint,
    delta: MouseScrollDelta,
) -> UiWindowPlatformInputEvent {
    let (scroll_delta, precise_scroll) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (
            y,
            zircon_runtime_interface::ui::dispatch::UiPreciseScrollDelta::lines(x, y),
        ),
        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
            let x = x as f32;
            let y = y as f32;
            (
                y * PIXEL_SCROLL_LINE_DELTA_SCALE,
                zircon_runtime_interface::ui::dispatch::UiPreciseScrollDelta::pixels(x, y),
            )
        }
    };
    UiWindowPlatformInputEvent::pointer(
        context,
        zircon_runtime_interface::ui::dispatch::UiPointerEvent::new(
            UiPointerEventKind::Scroll,
            point,
        )
        .with_scroll_delta(scroll_delta),
        Some(precise_scroll),
    )
}

pub(super) fn translate_pointer_button(
    context: UiWindowInputContext,
    state: ElementState,
    button: ButtonSource,
    position: PhysicalPosition<f64>,
) -> Option<UiWindowInputPumpEvent> {
    if let Some(finger_id) = touch_button_finger_id(&button) {
        let point = point_from_physical_position(position);
        let input = match state {
            ElementState::Pressed => {
                UiWindowPlatformInputEvent::touch_started(context, finger_id, point)
            }
            ElementState::Released => {
                UiWindowPlatformInputEvent::touch_ended(context, finger_id, point)
            }
        };
        return Some(input_event(input));
    }

    let button = pointer_button(button)?;
    let point = point_from_physical_position(position);
    let input = match state {
        ElementState::Pressed => {
            UiWindowPlatformInputEvent::mouse_button_down(context, button, point)
        }
        ElementState::Released => {
            UiWindowPlatformInputEvent::mouse_button_up(context, button, point)
        }
    };
    Some(input_event(input))
}

fn touch_button_finger_id(button: &ButtonSource) -> Option<UiPointerId> {
    match button {
        ButtonSource::Touch { finger_id, .. } => Some(pointer_id(*finger_id)),
        ButtonSource::Mouse(_) | ButtonSource::TabletTool { .. } | ButtonSource::Unknown(_) => None,
    }
}

fn pointer_button(button: ButtonSource) -> Option<UiPointerButton> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(UiPointerButton::Primary),
        Some(MouseButton::Right) => Some(UiPointerButton::Secondary),
        Some(MouseButton::Middle) => Some(UiPointerButton::Middle),
        _ => None,
    }
}

fn pointer_id(finger_id: FingerId) -> UiPointerId {
    UiPointerId::new(finger_id.into_raw() as u64)
}

fn point_from_physical_position<T>(position: PhysicalPosition<T>) -> UiPoint
where
    T: Into<f64>,
{
    UiPoint::new(position.x.into() as f32, position.y.into() as f32)
}
