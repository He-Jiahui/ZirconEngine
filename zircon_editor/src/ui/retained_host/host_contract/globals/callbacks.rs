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
pub(super) struct UiHostCallbacks {
    pub(super) frame_requested: Option<Callback0>,
    pub(super) close_prompt_action_clicked: Option<Callback1<SharedString>>,
    pub(super) menu_pointer_clicked: Option<Callback2<f32, f32>>,
    pub(super) menu_pointer_moved: Option<Callback2<f32, f32>>,
    pub(super) menu_pointer_scrolled: Option<Callback3<f32, f32, f32>>,
    pub(super) activity_rail_pointer_clicked: Option<Callback3<SharedString, f32, f32>>,
    pub(super) host_page_pointer_clicked: Option<Callback5<i32, f32, f32, f32, f32>>,
    pub(super) document_tab_pointer_clicked:
        Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    pub(super) document_tab_close_pointer_clicked:
        Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    pub(super) floating_window_header_pointer_clicked: Option<Callback2<f32, f32>>,
    pub(super) drawer_header_pointer_clicked:
        Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    pub(super) host_drag_pointer_event: Option<Callback3<i32, f32, f32>>,
    pub(super) host_resize_pointer_event: Option<Callback3<i32, f32, f32>>,
    pub(super) unhandled_keyboard_input: Option<Callback1<UiKeyboardInputEvent>>,
}

#[derive(Default)]
pub(super) struct PaneSurfaceCallbacks {
    pub(super) welcome_recent_pointer_clicked: Option<Callback4<f32, f32, f32, f32>>,
    pub(super) welcome_recent_pointer_moved: Option<Callback4<f32, f32, f32, f32>>,
    pub(super) welcome_recent_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(super) hierarchy_pointer_clicked: Option<Callback4<f32, f32, f32, f32>>,
    pub(super) hierarchy_pointer_moved: Option<Callback4<f32, f32, f32, f32>>,
    pub(super) hierarchy_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(super) hierarchy_pointer_event: Option<Callback6<i32, i32, f32, f32, f32, f32>>,
    pub(super) console_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(super) inspector_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(super) inspector_reference_pointer_event: Option<Callback6<i32, i32, f32, f32, f32, f32>>,
    pub(super) inspector_control_changed: Option<Callback2<SharedString, SharedString>>,
    pub(super) inspector_control_clicked: Option<Callback1<SharedString>>,
    pub(super) surface_control_clicked: Option<Callback2<SharedString, SharedString>>,
    pub(super) workbench_context_menu_requested: Option<Callback1<WorkbenchContextMenuRequestData>>,
    pub(super) surface_control_edited: Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(super) component_showcase_control_activated: Option<Callback2<SharedString, SharedString>>,
    pub(super) component_showcase_control_drag_delta:
        Option<Callback3<SharedString, SharedString, f32>>,
    pub(super) component_showcase_control_edited:
        Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(super) component_showcase_control_context_requested:
        Option<Callback4<SharedString, SharedString, f32, f32>>,
    pub(super) component_showcase_option_selected:
        Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(super) mesh_import_path_edited: Option<Callback1<SharedString>>,
    pub(super) asset_control_changed: Option<Callback3<SharedString, SharedString, SharedString>>,
    pub(super) asset_control_clicked: Option<Callback2<SharedString, SharedString>>,
    pub(super) asset_tree_pointer_clicked: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(super) asset_tree_pointer_moved: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(super) asset_tree_pointer_scrolled:
        Option<Callback6<SharedString, f32, f32, f32, f32, f32>>,
    pub(super) asset_content_pointer_clicked: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(super) asset_content_pointer_event:
        Option<Callback7<SharedString, i32, i32, f32, f32, f32, f32>>,
    pub(super) asset_content_pointer_moved: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(super) asset_content_pointer_scrolled:
        Option<Callback6<SharedString, f32, f32, f32, f32, f32>>,
    pub(super) asset_reference_pointer_clicked:
        Option<Callback6<SharedString, SharedString, f32, f32, f32, f32>>,
    pub(super) asset_reference_pointer_event:
        Option<Callback8<SharedString, SharedString, i32, i32, f32, f32, f32, f32>>,
    pub(super) asset_reference_pointer_moved:
        Option<Callback6<SharedString, SharedString, f32, f32, f32, f32>>,
    pub(super) asset_reference_pointer_scrolled:
        Option<Callback7<SharedString, SharedString, f32, f32, f32, f32, f32>>,
    pub(super) browser_asset_details_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    pub(super) welcome_control_changed: Option<Callback2<SharedString, SharedString>>,
    pub(super) welcome_control_clicked: Option<Callback1<SharedString>>,
    pub(super) viewport_pointer_event: Option<Callback5<i32, i32, f32, f32, f32>>,
    pub(super) viewport_toolbar_pointer_clicked:
        Option<Callback8<SharedString, SharedString, f32, f32, f32, f32, f32, f32>>,
    pub(super) ui_asset_action: Option<Callback2<SharedString, SharedString>>,
    pub(super) ui_asset_detail_event: Option<
        Callback6<SharedString, SharedString, SharedString, i32, SharedString, SharedString>,
    >,
    pub(super) ui_asset_collection_event:
        Option<Callback4<SharedString, SharedString, SharedString, i32>>,
}
