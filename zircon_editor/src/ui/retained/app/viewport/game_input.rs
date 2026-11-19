use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1,
};

use crate::core::play::PlayPreviewInputOwner;
use crate::ui::retained_host::host_contract::GameRmbInput;

use super::super::RetainedEditorHost;
use super::pointer_mapping::map_viewport_pointer_event;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn native_window_focus_lost(&mut self) {
        self.native_window_focus_lost_from(true);
    }

    pub(in crate::ui::retained_host::app) fn native_window_focus_lost_from(
        &mut self,
        game_root: bool,
    ) {
        if game_root && self.retire_game_rmb_on_focus_loss() {
            return;
        }
        // A different native window keeps the original focus notification, but it does not
        // consume the root window's pairing record.
        if !self.runtime.play_preview_input_active()
            || !(match self.runtime.play_preview_view_focused() {
                Ok(value) => value,
                Err(error) => {
                    self.set_status_line(error.to_string());
                    return;
                }
            })
        {
            return;
        }
        self.send_play_preview_background();
    }

    pub(in crate::ui::retained_host::app) fn route_play_preview_focus_lost(&mut self) {
        if !self.retire_game_rmb_on_focus_loss() {
            self.send_play_preview_background();
        }
    }

    fn retire_game_rmb_on_focus_loss(&mut self) -> bool {
        let Some(owner) = self.ui.take_game_rmb_owner() else {
            return false;
        };
        match self
            .runtime
            .route_owned_play_preview_input(play_background_event(), Some(&owner))
        {
            Ok(owner) => owner.is_some(),
            Err(error) => {
                self.set_status_line(error.to_string());
                true // An uncertain delivery is not replayed to the current runtime.
            }
        }
    }

    fn send_play_preview_background(&mut self) {
        if let Err(error) = self
            .runtime
            .route_play_preview_input(play_background_event())
        {
            self.set_status_line(error.to_string());
        }
    }

    pub(in crate::ui::retained_host::app) fn route_game_rmb(
        &mut self,
        input: GameRmbInput<'_>,
    ) -> Option<PlayPreviewInputOwner> {
        self.use_committed_pointer_layout();
        let (point, state, expected, inside) = match input {
            GameRmbInput::Down(point) => {
                self.focus_callback_source_window();
                (point, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, None, false)
            }
            GameRmbInput::Up {
                point,
                owner,
                inside,
            } => (
                point,
                ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
                Some(owner),
                inside,
            ),
        };
        let event = ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
            state,
            point.x,
            point.y,
        );
        match self.runtime.route_owned_play_preview_input(event, expected) {
            Ok(owner) => {
                if inside && owner.is_some() {
                    self.focus_callback_source_window();
                }
                owner
            }
            Err(error) => {
                self.set_status_line(error.to_string());
                None
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn game_viewport_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        delta: f32,
        _shift: bool,
        _control: bool,
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
        let runtime_event = match runtime_pointer_event(&event) {
            Ok(Some(event)) => event,
            Ok(None) => return,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        if let Err(error) = self.runtime.route_play_preview_input(runtime_event) {
            self.set_status_line(error.to_string());
        }
    }

    pub(in crate::ui::retained_host::app) fn route_focused_game_keyboard_input(
        &mut self,
        keyboard: &UiKeyboardInputEvent,
    ) -> bool {
        if !self.runtime.play_preview_input_active() {
            return false;
        }
        match self.runtime.play_preview_view_focused() {
            Ok(true) => {}
            Ok(false) => return false,
            Err(error) => {
                self.set_status_line(error.to_string());
                return true;
            }
        }
        let action = match keyboard.state {
            UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated => {
                ZR_RUNTIME_KEY_ACTION_PRESSED_V1
            }
            UiKeyboardInputState::Released => ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
        };
        let text = keyboard.text.as_deref().unwrap_or_default().as_bytes();
        let event = ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            action,
            keyboard.key_code,
            keyboard.scan_code.unwrap_or_default(),
            ZrByteSlice {
                data: text.as_ptr(),
                len: text.len(),
            },
        );
        match self.runtime.route_play_preview_input(event) {
            Ok(routed) => routed,
            Err(error) => {
                self.set_status_line(error.to_string());
                true
            }
        }
    }
}

fn play_background_event() -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
        ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    )
}

fn runtime_pointer_event(
    event: &zircon_runtime_interface::ui::dispatch::UiPointerEvent,
) -> Result<Option<ZrRuntimeEventV1>, String> {
    let viewport = ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1;
    let event = match event.kind {
        UiPointerEventKind::Move => ZrRuntimeEventV1::pointer_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            event.point.x,
            event.point.y,
        ),
        UiPointerEventKind::Down | UiPointerEventKind::Up => {
            let button = runtime_pointer_button(event.button.ok_or_else(|| {
                "game viewport mouse button event is missing its button".to_string()
            })?);
            let state = if event.kind == UiPointerEventKind::Down {
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1
            } else {
                ZR_RUNTIME_BUTTON_STATE_RELEASED_V1
            };
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                button,
                state,
                event.point.x,
                event.point.y,
            )
        }
        UiPointerEventKind::Scroll => ZrRuntimeEventV1::mouse_wheel_delta_at(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1,
            event.point.x,
            event.point.y,
            0.0,
            event.scroll_delta,
        ),
        UiPointerEventKind::Cancel => return Ok(None),
    };
    Ok(Some(event))
}

fn runtime_pointer_button(button: UiPointerButton) -> u32 {
    match button {
        UiPointerButton::Primary => ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        UiPointerButton::Secondary => ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
        UiPointerButton::Middle => ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    }
}
