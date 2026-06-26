use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use super::super::data::{
    HostDragStateData, HostMenuStateData, HostPageOverflowMenuStateData, HostResizeStateData,
    HostTextInputFocusData,
};
use super::state::{HostContractGlobal, HostContractState};

pub(crate) struct UiHostContext<'a> {
    state: Rc<RefCell<HostContractState>>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> HostContractGlobal for UiHostContext<'a> {
    fn from_state(state: Rc<RefCell<HostContractState>>) -> Self {
        Self {
            state,
            _lifetime: PhantomData,
        }
    }
}

impl UiHostContext<'_> {
    pub(crate) fn set_menu_state(&self, value: HostMenuStateData) {
        self.state.borrow_mut().menu_state = value;
    }

    pub(crate) fn set_host_page_overflow_menu_state(&self, value: HostPageOverflowMenuStateData) {
        self.state.borrow_mut().host_page_overflow_menu_state = value;
    }

    pub(crate) fn get_drag_state(&self) -> HostDragStateData {
        self.state.borrow().drag_state.clone()
    }

    pub(crate) fn set_drag_state(&self, value: HostDragStateData) {
        self.state.borrow_mut().drag_state = value;
    }

    pub(crate) fn get_resize_state(&self) -> HostResizeStateData {
        self.state.borrow().resize_state.clone()
    }

    pub(crate) fn set_resize_state(&self, value: HostResizeStateData) {
        self.state.borrow_mut().resize_state = value;
    }

    pub(crate) fn clear_resize_state(&self) {
        self.state.borrow_mut().resize_state = HostResizeStateData::default();
    }

    pub(crate) fn get_text_input_focus(&self) -> HostTextInputFocusData {
        self.state.borrow().text_input_focus.clone()
    }

    pub(crate) fn set_text_input_focus(&self, value: HostTextInputFocusData) {
        self.state.borrow_mut().text_input_focus = value;
    }

    pub(crate) fn clear_text_input_focus(&self) {
        self.state.borrow_mut().text_input_focus = HostTextInputFocusData::default();
    }

    callback_methods!(
        ui_callbacks,
        on_frame_requested,
        invoke_frame_requested,
        frame_requested,
        ()
    );
    callback_methods!(ui_callbacks, on_close_prompt_action_clicked, invoke_close_prompt_action_clicked, close_prompt_action_clicked, (action_id: SharedString));
    callback_methods!(ui_callbacks, on_menu_pointer_clicked, invoke_menu_pointer_clicked, menu_pointer_clicked, (x: f32, y: f32));
    callback_methods!(ui_callbacks, on_menu_pointer_moved, invoke_menu_pointer_moved, menu_pointer_moved, (x: f32, y: f32));
    callback_methods!(ui_callbacks, on_menu_pointer_scrolled, invoke_menu_pointer_scrolled, menu_pointer_scrolled, (x: f32, y: f32, delta: f32));
    callback_methods!(ui_callbacks, on_activity_rail_pointer_clicked, invoke_activity_rail_pointer_clicked, activity_rail_pointer_clicked, (side: SharedString, x: f32, y: f32));
    callback_methods!(ui_callbacks, on_host_page_pointer_clicked, invoke_host_page_pointer_clicked, host_page_pointer_clicked, (tab_index: i32, tab_x: f32, tab_width: f32, point_x: f32, point_y: f32));
    callback_methods!(ui_callbacks, on_document_tab_pointer_clicked, invoke_document_tab_pointer_clicked, document_tab_pointer_clicked, (surface_key: SharedString, tab_index: i32, tab_x: f32, tab_width: f32, point_x: f32, point_y: f32));
    callback_methods!(ui_callbacks, on_document_tab_close_pointer_clicked, invoke_document_tab_close_pointer_clicked, document_tab_close_pointer_clicked, (surface_key: SharedString, tab_index: i32, tab_x: f32, tab_width: f32, point_x: f32, point_y: f32));
    callback_methods!(ui_callbacks, on_floating_window_header_pointer_clicked, invoke_floating_window_header_pointer_clicked, floating_window_header_pointer_clicked, (x: f32, y: f32));
    callback_methods!(ui_callbacks, on_drawer_header_pointer_clicked, invoke_drawer_header_pointer_clicked, drawer_header_pointer_clicked, (surface_key: SharedString, tab_index: i32, tab_x: f32, tab_width: f32, point_x: f32, point_y: f32));
    callback_methods!(ui_callbacks, on_host_drag_pointer_event, invoke_host_drag_pointer_event, host_drag_pointer_event, (kind: i32, x: f32, y: f32));
    callback_methods!(ui_callbacks, on_host_resize_pointer_event, invoke_host_resize_pointer_event, host_resize_pointer_event, (kind: i32, x: f32, y: f32));
    callback_methods!(ui_callbacks, on_unhandled_keyboard_input, invoke_unhandled_keyboard_input, unhandled_keyboard_input, (keyboard: UiKeyboardInputEvent));
}
