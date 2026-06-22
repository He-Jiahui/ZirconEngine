use winit::event::WindowEvent;
use zircon_runtime::ui::platform_input::{translate_winit_modifiers, translate_winit_window_event};
use zircon_runtime_interface::ui::{
    dispatch::{UiInputEvent, UiKeyboardInputEvent, UiPointerInputEvent},
    layout::UiPoint,
    surface::UiPointerEventKind,
    window::{UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpEvent},
};

use super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(super) fn translate_platform_input_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<UiWindowInputPumpEvent> {
        let context = UiWindowInputContext {
            metadata: self.next_input_metadata(),
        }
        .with_modifiers(translate_winit_modifiers(self.current_modifiers));
        translate_winit_window_event(context, event)
    }
}

pub(super) fn event_uses_platform_input(event: &WindowEvent) -> bool {
    matches!(
        event,
        WindowEvent::PointerMoved { .. }
            | WindowEvent::PointerEntered { .. }
            | WindowEvent::PointerLeft { .. }
            | WindowEvent::PointerButton { .. }
            | WindowEvent::KeyboardInput { .. }
            | WindowEvent::Ime(_)
            | WindowEvent::MouseWheel { .. }
    )
}

pub(super) fn platform_keyboard_input(
    event: Option<UiWindowInputPumpEvent>,
) -> Option<UiKeyboardInputEvent> {
    match event? {
        UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(keyboard)) => Some(keyboard),
        _ => None,
    }
}

pub(super) fn platform_text_input(event: Option<UiWindowInputPumpEvent>) -> Option<String> {
    match event? {
        UiWindowInputPumpEvent::Input(UiInputEvent::Text(text)) => Some(text.text),
        _ => None,
    }
}

pub(super) fn platform_pointer_input(
    event: Option<UiWindowInputPumpEvent>,
) -> Option<UiPointerInputEvent> {
    match event? {
        UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) => Some(pointer),
        _ => None,
    }
}

pub(super) fn platform_pointer_move_point(
    event: Option<UiWindowInputPumpEvent>,
) -> Option<UiPoint> {
    match event? {
        UiWindowInputPumpEvent::Window(window) => match window.kind {
            UiWindowEventKind::CursorMoved { position, .. } => Some(position),
            _ => None,
        },
        UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer))
            if !pointer.metadata.pointer_source.is_touch_like()
                && matches!(pointer.event.kind, UiPointerEventKind::Move) =>
        {
            Some(pointer.event.point)
        }
        _ => None,
    }
}
