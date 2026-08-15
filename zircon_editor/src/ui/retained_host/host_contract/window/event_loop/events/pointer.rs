use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;
use zircon_runtime_interface::ui::surface::UiPointerEventKind;

use super::super::platform_input::PlatformInputTranslation;
use super::super::platform_input::{platform_pointer_input, platform_pointer_move_point};
use super::super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
    NativePointerButtonState,
};

impl UiHostWindowEventLoop {
    pub(super) fn handle_pointer_moved(
        &mut self,
        platform_event: PlatformInputTranslation,
        fallback_position: PhysicalPosition<f64>,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        let point = platform_pointer_move_point(platform_event.event).unwrap_or_else(|| {
            zircon_runtime_interface::ui::layout::UiPoint::new(
                fallback_position.x as f32,
                fallback_position.y as f32,
            )
        });
        self.last_pointer_position = Some((point.x, point.y));
        self.dispatch_pointer_result(dispatch_native_pointer_move(&self.host, point.x, point.y));
    }

    pub(super) fn handle_pointer_button(
        &mut self,
        platform_event: PlatformInputTranslation,
        fallback_position: PhysicalPosition<f64>,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        let Some(pointer) = platform_pointer_input(platform_event.event) else {
            self.reject_input_outcome();
            return;
        };
        if pointer.metadata.pointer_source.is_touch_like() {
            self.reject_input_outcome();
            return;
        }
        let Some(state) = pointer_button_state(pointer.event.kind) else {
            self.reject_input_outcome();
            return;
        };
        let point = pointer.event.point;
        let x = if point.x == 0.0 && point.y == 0.0 {
            fallback_position.x as f32
        } else {
            point.x
        };
        let y = if point.x == 0.0 && point.y == 0.0 {
            fallback_position.y as f32
        } else {
            point.y
        };
        self.last_pointer_position = Some((x, y));
        let result = dispatch_native_pointer_button(
            &self.host,
            state,
            pointer.event.button,
            pointer.metadata.modifiers,
            x,
            y,
        );
        self.dispatch_pointer_result(result);
        self.sync_ime_allowed();
    }

    pub(super) fn handle_mouse_wheel(
        &mut self,
        platform_event: PlatformInputTranslation,
        _fallback_delta: MouseScrollDelta,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        if let Some(pointer) = platform_pointer_input(platform_event.event) {
            if !matches!(pointer.event.kind, UiPointerEventKind::Scroll) {
                self.reject_input_outcome();
                return;
            }
            let (x, y) = self.last_pointer_position.unwrap_or((0.0, 0.0));
            self.dispatch_pointer_result(dispatch_native_pointer_scroll(
                &self.host,
                x,
                y,
                pointer.event.scroll_delta,
            ));
        } else {
            self.reject_input_outcome();
        }
    }
}

fn pointer_button_state(kind: UiPointerEventKind) -> Option<NativePointerButtonState> {
    match kind {
        UiPointerEventKind::Down => Some(NativePointerButtonState::Pressed),
        UiPointerEventKind::Up => Some(NativePointerButtonState::Released),
        _ => None,
    }
}
