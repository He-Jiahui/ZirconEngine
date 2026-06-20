use std::rc::Rc;

use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use super::super::data::WorkbenchContextMenuRequestData;

type Callback0 = Rc<dyn Fn()>;
type Callback1<A> = Rc<dyn Fn(A)>;
type Callback2<A, B> = Rc<dyn Fn(A, B)>;
type Callback3<A, B, C> = Rc<dyn Fn(A, B, C)>;
type Callback4<A, B, C, D> = Rc<dyn Fn(A, B, C, D)>;
type Callback5<A, B, C, D, E> = Rc<dyn Fn(A, B, C, D, E)>;
type Callback6<A, B, C, D, E, F> = Rc<dyn Fn(A, B, C, D, E, F)>;
type Callback7<A, B, C, D, E, F, G> = Rc<dyn Fn(A, B, C, D, E, F, G)>;
type Callback8<A, B, C, D, E, F, G, H> = Rc<dyn Fn(A, B, C, D, E, F, G, H)>;

#[derive(Default)]
pub(in crate::ui::retained_host::host_contract) struct UiHostCallbacks {
    pub(in crate::ui::retained_host::host_contract) frame_requested: Option<Callback0>,
    pub(in crate::ui::retained_host::host_contract) close_prompt_action_clicked:
        Option<Callback1<SharedString>>,
    pub(in crate::ui::retained_host::host_contract) menu_pointer_clicked:
        Option<Callback2<f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) menu_pointer_moved: Option<Callback2<f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) menu_pointer_scrolled:
        Option<Callback3<f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) activity_rail_pointer_clicked:
        Option<Callback3<SharedString, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) host_page_pointer_clicked:
        Option<Callback5<i32, f32, f32, f32, f32>>,
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
}

#[derive(Default)]
pub(in crate::ui::retained_host::host_contract) struct PaneSurfaceCallbacks {
    pub(in crate::ui::retained_host::host_contract) welcome_recent_pointer_clicked:
        Option<Callback4<f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) welcome_recent_pointer_moved:
        Option<Callback4<f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) welcome_recent_pointer_scrolled:
        Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) hierarchy_pointer_clicked:
        Option<Callback4<f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) hierarchy_pointer_moved:
        Option<Callback4<f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) hierarchy_pointer_scrolled:
        Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) hierarchy_pointer_event:
        Option<Callback6<i32, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) console_pointer_scrolled:
        Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) inspector_pointer_scrolled:
        Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) inspector_reference_pointer_event:
        Option<Callback6<i32, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) inspector_control_changed:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) inspector_control_clicked:
        Option<Callback1<SharedString>>,
    pub(in crate::ui::retained_host::host_contract) surface_control_clicked:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) workbench_context_menu_requested:
        Option<Callback1<WorkbenchContextMenuRequestData>>,
    pub(in crate::ui::retained_host::host_contract) surface_control_edited:
        Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) component_showcase_control_activated:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) component_showcase_control_drag_delta:
        Option<Callback3<SharedString, SharedString, f32>>,
    pub(in crate::ui::retained_host::host_contract) component_showcase_control_edited:
        Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) component_showcase_control_context_requested:
        Option<Callback4<SharedString, SharedString, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) component_showcase_option_selected:
        Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) mesh_import_path_edited:
        Option<Callback1<SharedString>>,
    pub(in crate::ui::retained_host::host_contract) asset_control_changed:
        Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) asset_control_clicked:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) asset_tree_pointer_clicked:
        Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_tree_pointer_moved:
        Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_tree_pointer_scrolled:
        Option<Callback6<SharedString, f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_content_pointer_clicked:
        Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_content_pointer_event:
        Option<Callback7<SharedString, i32, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_content_pointer_moved:
        Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_content_pointer_scrolled:
        Option<Callback6<SharedString, f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_reference_pointer_clicked:
        Option<Callback6<SharedString, SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_reference_pointer_event:
        Option<Callback8<SharedString, SharedString, i32, i32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_reference_pointer_moved:
        Option<Callback6<SharedString, SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) asset_reference_pointer_scrolled:
        Option<Callback7<SharedString, SharedString, f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) browser_asset_details_pointer_scrolled:
        Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) welcome_control_changed:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) welcome_control_clicked:
        Option<Callback1<SharedString>>,
    pub(in crate::ui::retained_host::host_contract) viewport_pointer_event:
        Option<Callback5<i32, i32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) viewport_toolbar_pointer_clicked:
        Option<Callback8<SharedString, SharedString, f32, f32, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) ui_asset_action:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) ui_asset_detail_event: Option<
        Callback6<SharedString, SharedString, SharedString, i32, SharedString, SharedString>,
    >,
    pub(in crate::ui::retained_host::host_contract) ui_asset_collection_event:
        Option<Callback4<SharedString, SharedString, SharedString, i32>>,
}
