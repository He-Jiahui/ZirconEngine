use zircon_runtime_interface::ui::dispatch::{
    UiImeDeleteSurrounding, UiImeInputEvent, UiImeInputEventKind, UiInputEvent,
    UiKeyboardInputEvent, UiKeyboardInputState, UiTextByteRange, UiTextInputEvent,
};
use zircon_runtime_interface::{
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1,
    ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1, ZR_RUNTIME_IME_STATE_DISABLED_V1,
    ZR_RUNTIME_IME_STATE_ENABLED_V1, ZR_RUNTIME_IME_STATE_PREEDIT_V1,
    ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1, ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1,
    ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_TEXT_V1, ZrRuntimeEventV1, ZrStatus,
};

use crate::core::framework::input::{
    ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit, InputEvent,
};

use super::super::RuntimeDynamicSession;
use super::super::input_events::{
    ime_cursor, ime_cursor_area, ime_surrounding_text, keyboard_logical_key,
};
use super::super::status::invalid_argument;
use super::event_payload;

impl RuntimeDynamicSession {
    pub(super) fn handle_keyboard(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = match event_payload(event) {
            Ok(payload) => payload,
            Err(status) => return status,
        };
        let text = if payload.is_empty() {
            None
        } else {
            String::from_utf8(payload.to_vec()).ok()
        };
        if event.button == ZR_RUNTIME_KEY_ACTION_TEXT_V1 {
            if let Some(text) = text {
                match self.dispatch_runtime_ui_event(|metadata| {
                    UiInputEvent::Text(UiTextInputEvent {
                        metadata,
                        text: text.clone(),
                    })
                }) {
                    Ok(true) => return ZrStatus::ok(),
                    Ok(false) => {}
                    Err(status) => return status,
                }
                self.submit_input_event(InputEvent::KeyboardInput {
                    key_code: event.key_code,
                    logical_key: None,
                    text: Some(text),
                    pressed: false,
                    repeat: false,
                });
            }
            return ZrStatus::ok();
        }

        let pressed = match event.button {
            ZR_RUNTIME_KEY_ACTION_PRESSED_V1 => true,
            ZR_RUNTIME_KEY_ACTION_RELEASED_V1 => false,
            _ => return ZrStatus::ok(),
        };
        let logical_key = keyboard_logical_key(event.key_code, text.as_deref());
        let ui_payload = (!self.runtime_ui.is_empty()).then(|| {
            (
                logical_key
                    .clone()
                    .unwrap_or_else(|| format!("Key{}", event.key_code)),
                text.clone(),
            )
        });
        if self.submit_input_event(InputEvent::KeyboardInput {
            key_code: event.key_code,
            logical_key,
            text,
            pressed,
            repeat: false,
        }) {
            if pressed {
                self.record_submitted_keyboard_press();
            } else {
                self.record_submitted_keyboard_release();
            }
        }
        if let Some((logical_key, text)) = ui_payload {
            match self.dispatch_runtime_ui_event(|metadata| {
                UiInputEvent::Keyboard(UiKeyboardInputEvent {
                    metadata,
                    state: if pressed {
                        UiKeyboardInputState::Pressed
                    } else {
                        UiKeyboardInputState::Released
                    },
                    key_code: event.key_code,
                    scan_code: Some(event.scan_code),
                    physical_key: format!("Key{}", event.key_code),
                    logical_key,
                    text,
                })
            }) {
                Ok(true) => return ZrStatus::ok(),
                Ok(false) => {}
                Err(status) => return status,
            }
        }
        ZrStatus::ok()
    }

    pub(super) fn handle_ime(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = match event_payload(event) {
            Ok(payload) => payload,
            Err(status) => return status,
        };
        let text_payload = || match String::from_utf8(payload.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Err(invalid_argument(b"invalid runtime ime payload")),
        };
        let input_event = match event.state {
            ZR_RUNTIME_IME_STATE_ENABLED_V1 => InputEvent::Ime(ImeEvent::Enabled),
            ZR_RUNTIME_IME_STATE_DISABLED_V1 => InputEvent::Ime(ImeEvent::Disabled),
            ZR_RUNTIME_IME_STATE_PREEDIT_V1 => InputEvent::Ime(ImeEvent::Preedit(ImePreedit {
                value: match text_payload() {
                    Ok(text) => text,
                    Err(status) => return status,
                },
                cursor: ime_cursor(event),
            })),
            ZR_RUNTIME_IME_STATE_COMMIT_V1 => match text_payload() {
                Ok(text) => InputEvent::Ime(ImeEvent::Commit(text)),
                Err(status) => return status,
            },
            ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1 => {
                InputEvent::Ime(ImeEvent::DeleteSurrounding(ImeDeleteSurrounding::new(
                    event.key_code as usize,
                    event.scan_code as usize,
                )))
            }
            ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1 => {
                InputEvent::ImeHostRequest(ImeHostRequest::Enable)
            }
            ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1 => {
                InputEvent::ImeHostRequest(ImeHostRequest::Disable)
            }
            ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1 => match ime_cursor_area(event) {
                Some(area) => InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(area)),
                None => return invalid_argument(b"invalid runtime ime cursor area"),
            },
            ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1 => {
                match ime_surrounding_text(event, payload) {
                    Ok(text) => {
                        InputEvent::ImeHostRequest(ImeHostRequest::SetSurroundingText(text))
                    }
                    Err(status) => return status,
                }
            }
            _ => return invalid_argument(b"unknown runtime ime state"),
        };
        let ui_dispatch = match &input_event {
            InputEvent::Ime(ImeEvent::Preedit(preedit)) => {
                self.dispatch_runtime_ui_event(|metadata| {
                    UiInputEvent::Ime(UiImeInputEvent {
                        metadata,
                        kind: UiImeInputEventKind::Preedit,
                        text: preedit.value.clone(),
                        cursor_range: preedit.cursor.and_then(|cursor| {
                            Some(UiTextByteRange::new(
                                u32::try_from(cursor.start).ok()?,
                                u32::try_from(cursor.end).ok()?,
                            ))
                        }),
                        preedit_clauses: Vec::new(),
                        delete_surrounding: None,
                    })
                })
            }
            InputEvent::Ime(ImeEvent::Commit(text)) => self.dispatch_runtime_ui_event(|metadata| {
                UiInputEvent::Ime(UiImeInputEvent {
                    metadata,
                    kind: UiImeInputEventKind::Commit,
                    text: text.clone(),
                    cursor_range: None,
                    preedit_clauses: Vec::new(),
                    delete_surrounding: None,
                })
            }),
            InputEvent::Ime(ImeEvent::DeleteSurrounding(delete)) => {
                self.dispatch_runtime_ui_event(|metadata| {
                    UiInputEvent::Ime(UiImeInputEvent {
                        metadata,
                        kind: UiImeInputEventKind::DeleteSurrounding,
                        text: String::new(),
                        cursor_range: None,
                        preedit_clauses: Vec::new(),
                        delete_surrounding: Some(UiImeDeleteSurrounding::new(
                            u32::try_from(delete.before_bytes).unwrap_or(u32::MAX),
                            u32::try_from(delete.after_bytes).unwrap_or(u32::MAX),
                        )),
                    })
                })
            }
            InputEvent::Ime(ImeEvent::Disabled) => self.dispatch_runtime_ui_event(|metadata| {
                UiInputEvent::Ime(UiImeInputEvent {
                    metadata,
                    kind: UiImeInputEventKind::Cancel,
                    text: String::new(),
                    cursor_range: None,
                    preedit_clauses: Vec::new(),
                    delete_surrounding: None,
                })
            }),
            _ => Ok(false),
        };
        match ui_dispatch {
            Ok(true) => return ZrStatus::ok(),
            Ok(false) => {}
            Err(status) => return status,
        }
        self.submit_input_event(input_event);
        ZrStatus::ok()
    }
}
