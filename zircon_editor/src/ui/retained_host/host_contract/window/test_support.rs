use winit::event::KeyEvent;
use winit::event::WindowEvent;
use winit::keyboard::ModifiersState;
use zircon_runtime::ui::platform_input::{translate_winit_modifiers, translate_winit_window_event};
use zircon_runtime_interface::ui::dispatch::{
    UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiWindowId,
};
use zircon_runtime_interface::ui::surface::UiPointerButton;
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
use super::constants::NATIVE_HOST_WINDOW_ID;
use super::UiHostWindow;

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
        let keyboard = native_keyboard_test_input(&event, modifiers);
        self.dispatch_keyboard_event(&event, keyboard)
    }

    pub(crate) fn dispatch_native_pointer_move_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_move(self, x, y)
    }

    pub(crate) fn dispatch_native_primary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Primary),
            Default::default(),
            x,
            y,
        )
    }

    pub(crate) fn dispatch_native_primary_release_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Released,
            Some(UiPointerButton::Primary),
            Default::default(),
            x,
            y,
        )
    }

    pub(crate) fn dispatch_native_secondary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Secondary),
            Default::default(),
            x,
            y,
        )
    }

    pub(crate) fn dispatch_native_middle_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Middle),
            Default::default(),
            x,
            y,
        )
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

fn native_keyboard_test_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
    metadata.window_id = Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID));
    metadata
}

fn native_keyboard_test_input(
    event: &KeyEvent,
    modifiers: ModifiersState,
) -> Option<zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent> {
    let platform_event = WindowEvent::KeyboardInput {
        device_id: None,
        event: event.clone(),
        is_synthetic: true,
    };
    let context = UiWindowInputContext {
        metadata: native_keyboard_test_metadata(),
    }
    .with_modifiers(translate_winit_modifiers(modifiers));
    let Some(UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(keyboard))) =
        translate_winit_window_event(context, &platform_event)
    else {
        return None;
    };
    Some(keyboard)
}
