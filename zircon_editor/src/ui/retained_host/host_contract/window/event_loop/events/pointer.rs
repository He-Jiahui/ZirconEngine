use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};

use super::super::platform_input::platform_pointer_input;
use super::super::platform_input::PlatformInputTranslation;
use super::super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
    NativePointerButtonState, WorkbenchTooltipPointerTarget,
};

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract::window::event_loop) fn handle_pointer_moved(
        &mut self,
        platform_event: PlatformInputTranslation,
        fallback_position: PhysicalPosition<f64>,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        let pointer = platform_pointer_input(platform_event.event);
        let point = pointer
            .as_ref()
            .map(|pointer| pointer.event.point)
            .unwrap_or_else(|| {
                zircon_runtime_interface::ui::layout::UiPoint::new(
                    fallback_position.x as f32,
                    fallback_position.y as f32,
                )
            });
        self.last_pointer_position = Some((point.x, point.y));
        let eligible = pointer
            .as_ref()
            .is_some_and(|pointer| !pointer.metadata.pointer_source.is_touch_like());
        self.host
            .global::<UiHostContext>()
            .invoke_workbench_pointer_move_pre_dispatch(
                pointer
                    .as_ref()
                    .and_then(|pointer| pointer.metadata.pointer_id)
                    .unwrap_or_default(),
                point.x,
                point.y,
                eligible,
            );
        let pointer = pointer.filter(|_| eligible);
        let native_pointer_id = pointer
            .as_ref()
            .map(|pointer| pointer.metadata.pointer_id.unwrap_or_default());
        let (result, tooltip_target): (_, Option<WorkbenchTooltipPointerTarget>) =
            dispatch_native_pointer_move(&self.host, native_pointer_id, point.x, point.y);
        if let Some(pointer) = pointer {
            self.host
                .global::<UiHostContext>()
                .invoke_workbench_pointer_input(pointer, tooltip_target);
        }
        self.dispatch_pointer_result(result);
    }

    pub(super) fn handle_pointer_button(
        &mut self,
        platform_event: PlatformInputTranslation,
        fallback_position: PhysicalPosition<f64>,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        let Some(mut pointer) = platform_pointer_input(platform_event.event) else {
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
        pointer.event.point.x = x;
        pointer.event.point.y = y;
        let button = pointer.event.button;
        let pointer_id = pointer.metadata.pointer_id.unwrap_or_default();
        let modifiers = pointer.metadata.modifiers;
        self.last_pointer_position = Some((x, y));
        self.host
            .global::<UiHostContext>()
            .invoke_workbench_pointer_input(pointer, None);
        let result =
            dispatch_native_pointer_button(&self.host, pointer_id, state, button, modifiers, x, y);
        if state == NativePointerButtonState::Released && button == Some(UiPointerButton::Primary) {
            self.host
                .global::<UiHostContext>()
                .invoke_workbench_primary_release_post_dispatch(pointer_id);
        }
        self.dispatch_pointer_result(result);
        self.sync_ime_allowed();
    }

    pub(super) fn handle_mouse_wheel(
        &mut self,
        platform_event: PlatformInputTranslation,
        _fallback_delta: MouseScrollDelta,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        if let Some(mut pointer) = platform_pointer_input(platform_event.event) {
            if !matches!(pointer.event.kind, UiPointerEventKind::Scroll) {
                self.reject_input_outcome();
                return;
            }
            let (x, y) = self.last_pointer_position.unwrap_or((0.0, 0.0));
            let scroll_delta = pointer.event.scroll_delta;
            pointer.event.point.x = x;
            pointer.event.point.y = y;
            self.host
                .global::<UiHostContext>()
                .invoke_workbench_pointer_input(pointer, None);
            self.dispatch_pointer_result(dispatch_native_pointer_scroll(
                &self.host,
                x,
                y,
                scroll_delta,
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
