use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use zircon_runtime_interface::ui::dispatch::{UiInputEvent, UiInputEventMetadata};

use super::super::data::HostTextInputFocusData;
use super::super::globals::{PaneSurfaceHostContext, UiHostContext};
use super::super::native_input_translation::native_keyboard_event_to_shared_input;
use super::super::native_keyboard::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
    workbench_popup_keyboard_command,
};
use super::super::redraw::NativePointerDispatchResult;
use super::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;

impl UiHostWindow {
    pub(crate) fn text_input_focus_active(&self) -> bool {
        self.state.borrow().text_input_focus.is_active()
    }

    pub(crate) fn dispatch_focused_text_insert(&self, text: &str) -> NativePointerDispatchResult {
        let text: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if text.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            value.push_str(&text);
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        self.dispatch_text_focus_value(focus.clone(), focus.edit_target_id(), value)
    }

    pub(crate) fn dispatch_focused_text_backspace(&self) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            if value.pop().is_none() {
                return NativePointerDispatchResult::idle();
            }
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        self.dispatch_text_focus_value(focus.clone(), focus.edit_target_id(), value)
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_key_event(
        &self,
        event: &KeyEvent,
    ) -> NativePointerDispatchResult {
        if event.state != ElementState::Pressed {
            return NativePointerDispatchResult::idle();
        }
        if !self.text_input_focus_active() {
            if let Some(command) = workbench_popup_keyboard_command(&event.logical_key) {
                let result = dispatch_workbench_popup_keyboard_command(self, command);
                if result.request_redraw() {
                    return result;
                }
            }
            if let Some(text) = event.text.as_deref() {
                let result = dispatch_workbench_popup_text_search(self, text);
                if result.request_redraw() {
                    return result;
                }
            }
        }
        match &event.logical_key {
            Key::Named(NamedKey::Backspace) => self.dispatch_focused_text_backspace(),
            Key::Named(NamedKey::Escape) => {
                self.global::<UiHostContext>().clear_text_input_focus();
                NativePointerDispatchResult::idle()
            }
            Key::Named(NamedKey::Enter) => self.dispatch_focused_text_commit(),
            _ => event
                .text
                .as_deref()
                .map_or_else(NativePointerDispatchResult::idle, |text| {
                    self.dispatch_focused_text_insert(text)
                }),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_native_keyboard_event(
        &self,
        event: &KeyEvent,
        modifiers: ModifiersState,
        metadata: UiInputEventMetadata,
        synthetic: bool,
    ) -> NativePointerDispatchResult {
        let text_focus_was_active = self.text_input_focus_active();
        let result = self.dispatch_focused_key_event(event);
        if !native_keyboard_event_consumed(text_focus_was_active, event, &result) {
            self.dispatch_unhandled_keyboard_input(event, modifiers, metadata, synthetic);
        }
        result
    }

    fn dispatch_unhandled_keyboard_input(
        &self,
        event: &KeyEvent,
        modifiers: ModifiersState,
        metadata: UiInputEventMetadata,
        synthetic: bool,
    ) {
        if event.state != ElementState::Pressed {
            return;
        }
        let UiInputEvent::Keyboard(keyboard) =
            native_keyboard_event_to_shared_input(metadata, event, modifiers, synthetic)
        else {
            return;
        };
        self.global::<UiHostContext>()
            .invoke_unhandled_keyboard_input(keyboard);
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_text_commit(
        &self,
    ) -> NativePointerDispatchResult {
        let focus = self.state.borrow().text_input_focus.clone();
        if !focus.is_active() || focus.commit_action_id.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        self.dispatch_text_focus_value(
            focus.clone(),
            focus.commit_target_id(),
            focus.value_text.to_string(),
        )
    }

    fn dispatch_text_focus_value(
        &self,
        focus: HostTextInputFocusData,
        target_id: SharedString,
        value: String,
    ) -> NativePointerDispatchResult {
        let value: SharedString = value.into();
        let control_id = focus.control_id.clone();
        let pane_host = self.global::<PaneSurfaceHostContext>();
        match focus.dispatch_kind.as_str() {
            "welcome_text" => pane_host.invoke_welcome_control_changed(target_id, value),
            "showcase" => {
                pane_host.invoke_component_showcase_control_edited(control_id, target_id, value)
            }
            "inspector" => pane_host.invoke_inspector_control_changed(control_id, value),
            kind if asset_dispatch_source(kind).is_some() => pane_host
                .invoke_asset_control_changed(
                    asset_dispatch_source(kind).unwrap_or("activity").into(),
                    control_id,
                    value,
                ),
            "commit_only" if target_id == focus.edit_target_id() => {
                return text_input_focus_redraw(&focus);
            }
            "commit_only" => pane_host.invoke_surface_control_edited(control_id, target_id, value),
            _ if !focus.edit_action_id.is_empty() => {
                pane_host.invoke_surface_control_edited(control_id, target_id, value)
            }
            _ => return NativePointerDispatchResult::idle(),
        }
        text_input_focus_redraw(&focus)
    }
}

fn asset_dispatch_source(dispatch_kind: &str) -> Option<&str> {
    if dispatch_kind == "asset" {
        return Some("activity");
    }
    dispatch_kind.strip_prefix("asset:")
}

fn native_keyboard_event_consumed(
    text_focus_was_active: bool,
    event: &KeyEvent,
    result: &NativePointerDispatchResult,
) -> bool {
    if result.request_redraw() {
        return true;
    }
    text_focus_was_active && text_focus_consumes_keyboard_event(event)
}

fn text_focus_consumes_keyboard_event(event: &KeyEvent) -> bool {
    if event.state != ElementState::Pressed {
        return false;
    }
    match &event.logical_key {
        Key::Named(NamedKey::Backspace | NamedKey::Escape | NamedKey::Enter) => true,
        _ => event
            .text
            .as_deref()
            .is_some_and(|text| text.chars().any(|ch| !ch.is_control())),
    }
}

fn text_input_focus_redraw(focus: &HostTextInputFocusData) -> NativePointerDispatchResult {
    let result = NativePointerDispatchResult::region(focus.edit_frame.clone());
    if result.request_redraw() {
        result
    } else {
        NativePointerDispatchResult::full_frame()
    }
}
