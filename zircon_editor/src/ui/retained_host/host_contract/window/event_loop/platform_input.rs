use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, PointerSource, WindowEvent};
use zircon_runtime::ui::platform_input::{translate_winit_modifiers, translate_winit_window_event};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiKeyboardInputEvent,
        UiPointerInputEvent,
    },
    layout::{UiPoint, UiSize},
    window::{UiWindowInputContext, UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelSize},
};

use super::UiHostWindowEventLoop;

pub(super) struct PlatformInputTranslation {
    pub(super) sequence: UiInputSequence,
    pub(super) event: Option<UiWindowInputPumpEvent>,
}

impl UiHostWindowEventLoop {
    pub(super) fn translate_platform_input_event(
        &mut self,
        event: &WindowEvent,
    ) -> PlatformInputTranslation {
        let metadata = self.next_input_metadata();
        self.translate_platform_input_event_with_metadata(metadata, event)
    }

    pub(super) fn translate_reserved_pointer_move_event(
        &self,
        mut metadata: UiInputEventMetadata,
        device_id: Option<DeviceId>,
        position: PhysicalPosition<f64>,
    ) -> PlatformInputTranslation {
        super::super::metadata::attach_native_window_id(&self.host, &mut metadata);
        let event = WindowEvent::PointerMoved {
            device_id,
            position,
            primary: true,
            source: PointerSource::Mouse,
        };
        self.translate_platform_input_event_with_metadata(metadata, &event)
    }

    fn translate_platform_input_event_with_metadata(
        &self,
        metadata: UiInputEventMetadata,
        event: &WindowEvent,
    ) -> PlatformInputTranslation {
        let sequence = metadata.sequence;
        let context = UiWindowInputContext {
            metadata,
            ..UiWindowInputContext::default()
        }
        .with_window_metrics(self.current_window_metrics())
        .with_modifiers(translate_winit_modifiers(self.current_modifiers));
        PlatformInputTranslation {
            sequence,
            event: translate_winit_window_event(context, event),
        }
    }

    fn current_window_metrics(&self) -> UiWindowMetrics {
        let window = self.host.window();
        let physical_size = window.size();
        let scale_factor = f64::from(window.scale_factor());
        UiWindowMetrics::new(
            UiSize::new(
                physical_size.width as f32 / scale_factor as f32,
                physical_size.height as f32 / scale_factor as f32,
            ),
            UiWindowPixelSize::new(physical_size.width, physical_size.height),
            scale_factor,
        )
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
            | WindowEvent::SurfaceResized(_)
            | WindowEvent::ScaleFactorChanged { .. }
    )
}

#[cfg(test)]
#[path = "tests/platform_input.rs"]
mod tests;

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
        UiWindowInputPumpEvent::Window(window) => match window.normalized_cursor_move_input()? {
            UiInputEvent::Pointer(pointer) => Some(pointer),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn platform_pointer_cancel_input(
    event: Option<UiWindowInputPumpEvent>,
    point: UiPoint,
) -> Option<UiPointerInputEvent> {
    let UiWindowInputPumpEvent::Window(window) = event? else {
        return None;
    };
    match window.normalized_pointer_cancel_input(point)? {
        UiInputEvent::Pointer(pointer) => Some(pointer),
        _ => None,
    }
}
