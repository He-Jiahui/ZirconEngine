use crate::ui::retained_host::host_contract::WorkbenchTooltipPointerTarget;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiPointerInputEvent};

use super::types::{Callback0, Callback1, Callback2, Callback3, Callback6};

#[derive(Default)]
pub(in crate::ui::retained_host::host_contract) struct UiHostCallbacks {
    pub(in crate::ui::retained_host::host_contract) frame_requested: Option<Callback0>,
    pub(in crate::ui::retained_host::host_contract) interactive_frame_requested: Option<Callback0>,
    pub(in crate::ui::retained_host::host_contract) workbench_pointer_input:
        Option<Callback2<UiPointerInputEvent, Option<WorkbenchTooltipPointerTarget>>>,
    pub(in crate::ui::retained_host::host_contract) workbench_input_activity: Option<Callback0>,
    pub(in crate::ui::retained_host::host_contract) asset_deletion_blocker_closed:
        Option<Callback0>,
    pub(in crate::ui::retained_host::host_contract) close_prompt_action_clicked:
        Option<Callback1<SharedString>>,
    pub(in crate::ui::retained_host::host_contract) menu_pointer_clicked:
        Option<Callback2<f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) menu_pointer_moved: Option<Callback2<f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) menu_pointer_scrolled:
        Option<Callback3<f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) settings_window_scrolled:
        Option<Callback2<f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) activity_rail_pointer_clicked:
        Option<Callback3<SharedString, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) host_page_pointer_clicked:
        Option<Callback2<i32, bool>>,
    pub(in crate::ui::retained_host::host_contract) document_tab_pointer_clicked:
        Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) document_tab_close_pointer_clicked:
        Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) floating_window_header_pointer_clicked:
        Option<Callback2<f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) drawer_header_pointer_clicked:
        Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) host_drag_pointer_event:
        Option<Callback3<i32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) host_resize_pointer_event:
        Option<Callback3<i32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) unhandled_keyboard_input:
        Option<Callback1<UiKeyboardInputEvent>>,
    pub(in crate::ui::retained_host::host_contract) native_window_focus_lost: Option<Callback0>,
}
