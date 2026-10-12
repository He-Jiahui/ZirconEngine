use winit::event::WindowEvent;
use winit::event::{KeyEvent, PointerKind};
use winit::keyboard::ModifiersState;
use zircon_runtime::ui::platform_input::{translate_winit_modifiers, translate_winit_window_event};
use zircon_runtime_interface::ui::dispatch::{
    UiInputEvent, UiInputEventMetadata, UiPointerEvent, UiPointerId, UiPointerInputEvent,
    UiPointerSource,
};
use zircon_runtime_interface::ui::layout::UiPoint;
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::ui::window::{UiWindowInputContext, UiWindowInputPumpEvent};

use super::super::native_keyboard::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
    WorkbenchPopupKeyboardCommand,
};
use super::super::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
    NativePointerButtonState,
};
use super::super::redraw::NativePointerDispatchResult;
use super::UiHostWindow;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::primitives::CloseRequestResponse;

impl UiHostWindow {
    pub(crate) fn request_host_frame_for_test(&self) {
        self.request_frame_update();
    }

    pub(crate) fn presentation_rebuild_count_for_test(&self) -> u64 {
        self.state.borrow().presentation_rebuild_count
    }

    pub(crate) fn dispatch_native_key_for_test(
        &self,
        event: KeyEvent,
        modifiers: ModifiersState,
    ) -> NativePointerDispatchResult {
        let keyboard = native_keyboard_test_input(self, &event, modifiers);
        self.dispatch_keyboard_event(&event, keyboard)
    }

    pub(crate) fn dispatch_native_pointer_move_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_pointer_move_with_pointer_for_test(UiPointerId::default(), x, y)
    }

    pub(crate) fn dispatch_native_pointer_move_with_pointer_for_test(
        &self,
        pointer_id: UiPointerId,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_pointer_move_input_for_test(native_pointer_test_input(
            self,
            UiPointerEventKind::Move,
            None,
            pointer_id,
            x,
            y,
        ))
    }

    pub(crate) fn dispatch_native_touch_like_pointer_move_for_test(
        &self,
        pointer_id: UiPointerId,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        let mut pointer =
            native_pointer_test_input(self, UiPointerEventKind::Move, None, pointer_id, x, y);
        pointer.metadata.pointer_source = UiPointerSource::Touch;
        self.dispatch_native_pointer_move_input_for_test(pointer)
    }

    pub(crate) fn dispatch_native_untranslated_pointer_move_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.global::<UiHostContext>()
            .invoke_workbench_pointer_move_pre_dispatch(UiPointerId::default(), x, y, false);
        dispatch_native_pointer_move(self, None, x, y).0
    }

    fn dispatch_native_pointer_move_input_for_test(
        &self,
        pointer: UiPointerInputEvent,
    ) -> NativePointerDispatchResult {
        let x = pointer.event.point.x;
        let y = pointer.event.point.y;
        let eligible = !pointer.metadata.pointer_source.is_touch_like();
        self.global::<UiHostContext>()
            .invoke_workbench_pointer_move_pre_dispatch(
                pointer.metadata.pointer_id.unwrap_or_default(),
                x,
                y,
                eligible,
            );
        let native_pointer_id = eligible.then(|| pointer.metadata.pointer_id.unwrap_or_default());
        let (result, tooltip_target) = dispatch_native_pointer_move(self, native_pointer_id, x, y);
        if eligible {
            self.global::<UiHostContext>()
                .invoke_workbench_pointer_input(pointer, tooltip_target);
        }
        result
    }

    pub(crate) fn dispatch_native_primary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_primary_press_with_pointer_for_test(UiPointerId::default(), x, y)
    }

    pub(crate) fn dispatch_native_primary_press_with_pointer_for_test(
        &self,
        pointer_id: UiPointerId,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_button_with_pointer_for_test(
            NativePointerButtonState::Pressed,
            UiPointerButton::Primary,
            pointer_id,
            x,
            y,
        )
    }

    pub(crate) fn dispatch_native_primary_release_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_primary_release_with_pointer_for_test(UiPointerId::default(), x, y)
    }

    pub(crate) fn dispatch_native_primary_release_with_pointer_for_test(
        &self,
        pointer_id: UiPointerId,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_button_with_pointer_for_test(
            NativePointerButtonState::Released,
            UiPointerButton::Primary,
            pointer_id,
            x,
            y,
        )
    }

    pub(crate) fn dispatch_native_secondary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_button_with_pointer_for_test(
            NativePointerButtonState::Pressed,
            UiPointerButton::Secondary,
            UiPointerId::default(),
            x,
            y,
        )
    }

    pub(crate) fn dispatch_native_middle_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        self.dispatch_native_button_with_pointer_for_test(
            NativePointerButtonState::Pressed,
            UiPointerButton::Middle,
            UiPointerId::default(),
            x,
            y,
        )
    }

    fn dispatch_native_button_with_pointer_for_test(
        &self,
        state: NativePointerButtonState,
        button: UiPointerButton,
        pointer_id: UiPointerId,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        let kind = match state {
            NativePointerButtonState::Pressed => UiPointerEventKind::Down,
            NativePointerButtonState::Released => UiPointerEventKind::Up,
        };
        self.global::<UiHostContext>()
            .invoke_workbench_pointer_input(
                native_pointer_test_input(self, kind, Some(button), pointer_id, x, y),
                None,
            );
        let result = dispatch_native_pointer_button(
            self,
            pointer_id,
            state,
            Some(button),
            Default::default(),
            x,
            y,
        );
        if state == NativePointerButtonState::Released && button == UiPointerButton::Primary {
            self.global::<UiHostContext>()
                .invoke_workbench_primary_release_post_dispatch(pointer_id);
        }
        result
    }

    pub(crate) fn dispatch_native_close_request_for_test(&self) -> CloseRequestResponse {
        self.dispatch_native_close_request_event_for_test()
    }

    pub(crate) fn dispatch_native_focus_lost_for_test(&self) {
        self.dispatch_native_focus_lost_event_for_test();
    }

    pub(crate) fn dispatch_native_pointer_leave_for_test(&self, kind: PointerKind, x: f32, y: f32) {
        self.dispatch_native_pointer_leave_event_for_test(kind, x, y);
    }

    pub(crate) fn dispatch_native_pointer_cancel_for_test(&self, x: f32, y: f32) {
        self.global::<UiHostContext>()
            .invoke_workbench_pointer_input(
                native_pointer_test_input(
                    self,
                    UiPointerEventKind::Cancel,
                    None,
                    UiPointerId::default(),
                    x,
                    y,
                ),
                None,
            );
    }

    pub(crate) fn dispatch_native_pointer_scroll_for_test(
        &self,
        x: f32,
        y: f32,
        delta: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_scroll(self, x, y, delta)
    }

    pub(crate) fn dispatch_native_text_input_for_test(
        &self,
        text: &str,
    ) -> NativePointerDispatchResult {
        self.dispatch_focused_text_insert(text)
    }

    pub(crate) fn dispatch_native_text_for_test(&self, text: &str) -> NativePointerDispatchResult {
        self.dispatch_native_text_input_for_test(text)
    }

    pub(crate) fn dispatch_native_backspace_for_test(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_backspace()
    }

    pub(crate) fn dispatch_native_enter_for_test(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_commit()
    }

    pub(crate) fn dispatch_native_popup_arrow_down_for_test(&self) -> NativePointerDispatchResult {
        dispatch_workbench_popup_keyboard_command(self, WorkbenchPopupKeyboardCommand::Next)
    }

    pub(crate) fn dispatch_native_popup_arrow_up_for_test(&self) -> NativePointerDispatchResult {
        dispatch_workbench_popup_keyboard_command(self, WorkbenchPopupKeyboardCommand::Previous)
    }

    pub(crate) fn dispatch_native_popup_home_for_test(&self) -> NativePointerDispatchResult {
        dispatch_workbench_popup_keyboard_command(self, WorkbenchPopupKeyboardCommand::First)
    }

    pub(crate) fn dispatch_native_popup_end_for_test(&self) -> NativePointerDispatchResult {
        dispatch_workbench_popup_keyboard_command(self, WorkbenchPopupKeyboardCommand::Last)
    }

    pub(crate) fn dispatch_native_popup_text_for_test(
        &self,
        text: &str,
    ) -> NativePointerDispatchResult {
        dispatch_workbench_popup_text_search(self, text)
    }

    pub(crate) fn dispatch_native_popup_enter_for_test(&self) -> NativePointerDispatchResult {
        dispatch_workbench_popup_keyboard_command(self, WorkbenchPopupKeyboardCommand::Accept)
    }

    pub(crate) fn dispatch_native_popup_escape_for_test(&self) -> NativePointerDispatchResult {
        dispatch_workbench_popup_keyboard_command(self, WorkbenchPopupKeyboardCommand::Cancel)
    }
}

fn native_keyboard_test_metadata(ui: &UiHostWindow) -> UiInputEventMetadata {
    super::metadata::native_input_metadata(ui, 1)
}

fn native_keyboard_test_input(
    ui: &UiHostWindow,
    event: &KeyEvent,
    modifiers: ModifiersState,
) -> Option<zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent> {
    let platform_event = WindowEvent::KeyboardInput {
        device_id: None,
        event: event.clone(),
        is_synthetic: true,
    };
    let context = UiWindowInputContext {
        metadata: native_keyboard_test_metadata(ui),
        ..UiWindowInputContext::default()
    }
    .with_modifiers(translate_winit_modifiers(modifiers));
    let Some(UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(keyboard))) =
        translate_winit_window_event(context, &platform_event)
    else {
        return None;
    };
    Some(keyboard)
}

fn native_pointer_test_input(
    ui: &UiHostWindow,
    kind: UiPointerEventKind,
    button: Option<UiPointerButton>,
    pointer_id: UiPointerId,
    x: f32,
    y: f32,
) -> UiPointerInputEvent {
    let mut metadata = super::metadata::native_input_metadata(ui, 1);
    metadata.pointer_id = Some(pointer_id);
    let mut event = UiPointerEvent::new(kind, UiPoint::new(x, y));
    if let Some(button) = button {
        event = event.with_button(button);
    }
    UiPointerInputEvent {
        metadata,
        event,
        precise_scroll: None,
    }
}
