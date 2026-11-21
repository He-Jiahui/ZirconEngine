#[path = "edit/dispatch.rs"]
mod dispatch;
#[path = "edit/redraw.rs"]
mod redraw;

use std::sync::Arc;

use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::primitives::SharedString;
use dispatch::dispatch_text_focus_value;

struct DetailsAxisSubmitScope<'a>(&'a UiHostWindow);

impl Drop for DetailsAxisSubmitScope<'_> {
    fn drop(&mut self) {
        self.0.state.borrow_mut().details_axis_submit_in_flight = None;
    }
}

impl UiHostWindow {
    pub(crate) fn text_input_focus_active(&self) -> bool {
        self.state.borrow().text_input_focus.is_active()
    }

    pub(crate) fn text_input_focus_accepts_text(&self) -> bool {
        self.state.borrow().text_input_focus.accepts_text_input()
    }

    pub(crate) fn chord_capture_focus_active(&self) -> bool {
        self.state
            .borrow()
            .text_input_focus
            .captures_keyboard_chord()
    }

    pub(crate) fn dispatch_focused_text_insert(&self, text: &str) -> NativePointerDispatchResult {
        self.dispatch_focused_text_edit(|focus| focus.insert_text(text))
    }

    pub(crate) fn dispatch_focused_text_backspace(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_edit(|focus| focus.delete_text(true))
    }

    pub(crate) fn dispatch_focused_text_delete(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_edit(|focus| focus.delete_text(false))
    }

    fn dispatch_focused_text_edit(
        &self,
        edit: impl FnOnce(&mut HostTextInputFocusData) -> bool,
    ) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let mut focus = state.text_input_focus.as_ref().clone();
            if !focus.accepts_text_input() || !edit(&mut focus) {
                return NativePointerDispatchResult::idle();
            }
            if focus.is_details_transform_axis() {
                focus.details_axis_edited = true;
            }
            let value = focus.value_text.clone();
            state.replace_text_input_focus(focus.clone());
            (focus, value)
        };
        let target_id = focus.edit_target_id();
        dispatch_text_focus_value(self, focus, target_id, value)
    }

    pub(crate) fn dispatch_focused_text_caret(
        &self,
        movement: &str,
        extend: bool,
    ) -> NativePointerDispatchResult {
        let mut state = self.state.borrow_mut();
        let mut focus = state.text_input_focus.as_ref().clone();
        if !focus.accepts_text_input() || !focus.move_text_caret(movement, extend) {
            return NativePointerDispatchResult::idle();
        }
        let result = redraw::text_input_focus_redraw(&focus);
        state.replace_text_input_focus(focus);
        result
    }

    pub(crate) fn cancel_focused_text_input(&self) -> NativePointerDispatchResult {
        let mut state = self.state.borrow_mut();
        let focus = state.text_input_focus.as_ref().clone();
        if !focus.is_active() {
            return NativePointerDispatchResult::idle();
        }
        let result = redraw::text_input_focus_redraw(&focus);
        state.replace_text_input_focus(Default::default());
        result
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_text_commit(
        &self,
    ) -> NativePointerDispatchResult {
        let state = self.state.borrow();
        let focus = &state.text_input_focus;
        if !focus.is_active() || focus.commit_action_id.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        if focus.is_details_transform_axis() {
            if !focus.details_axis_edited || state.details_axis_submit_in_flight.is_some() {
                // The keyboard router still consumes Enter for the focused control.
                return NativePointerDispatchResult::idle();
            }
            let submitted = Arc::clone(focus);
            let focus = submitted.as_ref().clone();
            drop(state);
            self.state.borrow_mut().details_axis_submit_in_flight = Some(submitted);
            let _scope = DetailsAxisSubmitScope(self);
            let target_id = focus.commit_target_id();
            let value = focus.value_text.clone();
            return dispatch_text_focus_value(self, focus, target_id, value);
        }
        let focus = focus.as_ref().clone();
        drop(state);
        let target_id = focus.commit_target_id();
        let value = focus.value_text.clone();
        dispatch_text_focus_value(self, focus, target_id, value)
    }

    /// Acknowledge only the successful axis Submit from this native focus snapshot.
    /// A rejecting or skipped handler leaves the draft and its edit intent retryable.
    pub(crate) fn acknowledge_details_axis_text_commit(
        &self,
        control_id: &str,
        action_id: &str,
        value: &str,
    ) {
        let mut state = self.state.borrow_mut();
        let Some(submitted) = state.details_axis_submit_in_flight.as_ref() else {
            return;
        };
        if !Arc::ptr_eq(submitted, &state.text_input_focus)
            || submitted.control_id.as_str() != control_id
            || submitted.commit_action_id.as_str() != action_id
            || submitted.value_text.as_str() != value
        {
            return;
        }
        let mut focus = submitted.as_ref().clone();
        focus.details_axis_edited = false;
        state.replace_text_input_focus(focus);
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_chord_commit(
        &self,
        value: SharedString,
    ) -> NativePointerDispatchResult {
        let focus = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.as_ref().clone();
            if !focus.captures_keyboard_chord() || focus.commit_action_id.is_empty() {
                return NativePointerDispatchResult::idle();
            }
            state.replace_text_input_focus(Default::default());
            focus
        };
        let target_id = focus.commit_target_id();
        dispatch_text_focus_value(self, focus, target_id, value)
    }

    pub(in crate::ui::retained_host::host_contract) fn cancel_focused_chord_capture(
        &self,
    ) -> NativePointerDispatchResult {
        let focus = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.as_ref().clone();
            if !focus.captures_keyboard_chord() {
                return NativePointerDispatchResult::idle();
            }
            state.replace_text_input_focus(Default::default());
            focus
        };
        let result = NativePointerDispatchResult::region(focus.edit_frame);
        if result.request_redraw() {
            result
        } else {
            NativePointerDispatchResult::full_frame()
        }
    }
}
