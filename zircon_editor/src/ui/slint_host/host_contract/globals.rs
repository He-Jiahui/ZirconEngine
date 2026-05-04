use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use slint::{CloseRequestResponse, Image, ModelRc, PhysicalPosition, PhysicalSize, SharedString};

use super::data::{
    AssetFolderData, AssetItemData, AssetReferenceData, AssetSelectionData, HostDragStateData,
    HostMenuStateData, HostWindowPresentationData, ProjectOverviewData, RecentProjectData,
    WelcomePaneData,
};

type Callback0 = Rc<dyn Fn()>;
type Callback1<A> = Rc<dyn Fn(A)>;
type Callback2<A, B> = Rc<dyn Fn(A, B)>;
type Callback3<A, B, C> = Rc<dyn Fn(A, B, C)>;
type Callback4<A, B, C, D> = Rc<dyn Fn(A, B, C, D)>;
type Callback5<A, B, C, D, E> = Rc<dyn Fn(A, B, C, D, E)>;
type Callback6<A, B, C, D, E, F> = Rc<dyn Fn(A, B, C, D, E, F)>;
type Callback8<A, B, C, D, E, F, G, H> = Rc<dyn Fn(A, B, C, D, E, F, G, H)>;

pub(crate) trait HostContractGlobal: Sized {
    fn from_state(state: Rc<RefCell<HostContractState>>) -> Self;
}

pub(crate) struct HostContractState {
    pub(crate) window_position: PhysicalPosition,
    pub(crate) window_size: PhysicalSize,
    pub(crate) window_visible: bool,
    pub(crate) window_maximized: bool,
    pub(crate) close_requested: Option<Rc<dyn Fn() -> CloseRequestResponse>>,
    pub(crate) host_presentation: HostWindowPresentationData,
    pub(crate) menu_state: HostMenuStateData,
    pub(crate) drag_state: HostDragStateData,
    pub(crate) welcome_pane: WelcomePaneData,
    ui_callbacks: UiHostCallbacks,
    pane_callbacks: PaneSurfaceCallbacks,
}

impl HostContractState {
    pub(crate) fn new(window_size: PhysicalSize) -> Self {
        Self {
            window_position: PhysicalPosition::new(0, 0),
            window_size,
            window_visible: false,
            window_maximized: false,
            close_requested: None,
            host_presentation: HostWindowPresentationData::default(),
            menu_state: HostMenuStateData::default(),
            drag_state: HostDragStateData::default(),
            welcome_pane: WelcomePaneData::default(),
            ui_callbacks: UiHostCallbacks::default(),
            pane_callbacks: PaneSurfaceCallbacks::default(),
        }
    }
}

#[derive(Default)]
struct UiHostCallbacks {
    frame_requested: Option<Callback0>,
    menu_pointer_clicked: Option<Callback2<f32, f32>>,
    menu_pointer_moved: Option<Callback2<f32, f32>>,
    menu_pointer_scrolled: Option<Callback3<f32, f32, f32>>,
    activity_rail_pointer_clicked: Option<Callback3<SharedString, f32, f32>>,
    host_page_pointer_clicked: Option<Callback5<i32, f32, f32, f32, f32>>,
    document_tab_pointer_clicked: Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    document_tab_close_pointer_clicked: Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    floating_window_header_pointer_clicked: Option<Callback2<f32, f32>>,
    drawer_header_pointer_clicked: Option<Callback6<SharedString, i32, f32, f32, f32, f32>>,
    host_drag_pointer_event: Option<Callback3<i32, f32, f32>>,
    host_resize_pointer_event: Option<Callback3<i32, f32, f32>>,
}

#[derive(Default)]
struct PaneSurfaceCallbacks {
    welcome_recent_pointer_clicked: Option<Callback4<f32, f32, f32, f32>>,
    welcome_recent_pointer_moved: Option<Callback4<f32, f32, f32, f32>>,
    welcome_recent_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    hierarchy_pointer_clicked: Option<Callback4<f32, f32, f32, f32>>,
    hierarchy_pointer_moved: Option<Callback4<f32, f32, f32, f32>>,
    hierarchy_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    hierarchy_pointer_event: Option<Callback6<i32, i32, f32, f32, f32, f32>>,
    console_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    inspector_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    inspector_reference_pointer_event: Option<Callback6<i32, i32, f32, f32, f32, f32>>,
    inspector_control_changed: Option<Callback2<SharedString, SharedString>>,
    inspector_control_clicked: Option<Callback1<SharedString>>,
    surface_control_clicked: Option<Callback2<SharedString, SharedString>>,
    component_showcase_control_activated: Option<Callback2<SharedString, SharedString>>,
    component_showcase_control_drag_delta: Option<Callback3<SharedString, SharedString, f32>>,
    component_showcase_control_edited: Option<Callback3<SharedString, SharedString, SharedString>>,
    component_showcase_control_context_requested:
        Option<Callback4<SharedString, SharedString, f32, f32>>,
    component_showcase_option_selected: Option<Callback3<SharedString, SharedString, SharedString>>,
    mesh_import_path_edited: Option<Callback1<SharedString>>,
    asset_control_changed: Option<Callback3<SharedString, SharedString, SharedString>>,
    asset_control_clicked: Option<Callback2<SharedString, SharedString>>,
    asset_tree_pointer_clicked: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    asset_tree_pointer_moved: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    asset_tree_pointer_scrolled: Option<Callback6<SharedString, f32, f32, f32, f32, f32>>,
    asset_content_pointer_clicked: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    asset_content_pointer_event: Option<Callback7<SharedString, i32, i32, f32, f32, f32, f32>>,
    asset_content_pointer_moved: Option<Callback5<SharedString, f32, f32, f32, f32>>,
    asset_content_pointer_scrolled: Option<Callback6<SharedString, f32, f32, f32, f32, f32>>,
    asset_reference_pointer_clicked:
        Option<Callback6<SharedString, SharedString, f32, f32, f32, f32>>,
    asset_reference_pointer_event:
        Option<Callback8<SharedString, SharedString, i32, i32, f32, f32, f32, f32>>,
    asset_reference_pointer_moved:
        Option<Callback6<SharedString, SharedString, f32, f32, f32, f32>>,
    asset_reference_pointer_scrolled:
        Option<Callback7<SharedString, SharedString, f32, f32, f32, f32, f32>>,
    browser_asset_details_pointer_scrolled: Option<Callback5<f32, f32, f32, f32, f32>>,
    welcome_control_changed: Option<Callback2<SharedString, SharedString>>,
    welcome_control_clicked: Option<Callback1<SharedString>>,
    viewport_pointer_event: Option<Callback5<i32, i32, f32, f32, f32>>,
    viewport_toolbar_pointer_clicked:
        Option<Callback8<SharedString, SharedString, f32, f32, f32, f32, f32, f32>>,
    ui_asset_action: Option<Callback2<SharedString, SharedString>>,
    ui_asset_detail_event: Option<
        Callback6<SharedString, SharedString, SharedString, i32, SharedString, SharedString>,
    >,
    ui_asset_collection_event: Option<Callback4<SharedString, SharedString, SharedString, i32>>,
}

type Callback7<A, B, C, D, E, F, G> = Rc<dyn Fn(A, B, C, D, E, F, G)>;

macro_rules! callback_methods {
    ($callbacks:ident, $on_name:ident, $invoke_name:ident, $field:ident, ($($arg:ident : $ty:ty),* $(,)?)) => {
        pub(crate) fn $on_name(&self, callback: impl Fn($($ty),*) + 'static) {
            self.state.borrow_mut().$callbacks.$field = Some(Rc::new(callback));
        }

        pub(crate) fn $invoke_name(&self, $($arg: $ty),*) {
            let callback = self.state.borrow().$callbacks.$field.clone();
            if let Some(callback) = callback {
                callback($($arg),*);
            }
        }
    };
}

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

    pub(crate) fn get_drag_state(&self) -> HostDragStateData {
        self.state.borrow().drag_state.clone()
    }

    pub(crate) fn set_drag_state(&self, value: HostDragStateData) {
        self.state.borrow_mut().drag_state = value;
    }

    callback_methods!(ui_callbacks, on_frame_requested, invoke_frame_requested, frame_requested, ());
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
}

pub(crate) struct PaneSurfaceHostContext<'a> {
    state: Rc<RefCell<HostContractState>>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> HostContractGlobal for PaneSurfaceHostContext<'a> {
    fn from_state(state: Rc<RefCell<HostContractState>>) -> Self {
        Self {
            state,
            _lifetime: PhantomData,
        }
    }
}

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_recent_projects(&self, _value: ModelRc<RecentProjectData>) {}
    pub(crate) fn set_project_overview(&self, _value: ProjectOverviewData) {}
    pub(crate) fn set_activity_asset_tree_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_activity_asset_content_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_activity_asset_content_items(&self, _value: ModelRc<AssetItemData>) {}
    pub(crate) fn set_activity_asset_selection(&self, _value: AssetSelectionData) {}
    pub(crate) fn set_activity_asset_references(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_activity_asset_used_by(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_activity_asset_search_query(&self, _value: SharedString) {}
    pub(crate) fn set_activity_asset_kind_filter(&self, _value: SharedString) {}
    pub(crate) fn set_activity_asset_view_mode(&self, _value: SharedString) {}
    pub(crate) fn set_activity_asset_utility_tab(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_tree_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_browser_asset_content_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_browser_asset_content_items(&self, _value: ModelRc<AssetItemData>) {}
    pub(crate) fn set_browser_asset_selection(&self, _value: AssetSelectionData) {}
    pub(crate) fn set_browser_asset_references(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_browser_asset_used_by(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_browser_asset_search_query(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_kind_filter(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_view_mode(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_utility_tab(&self, _value: SharedString) {}
    pub(crate) fn set_welcome_pane(&self, value: WelcomePaneData) {
        self.state.borrow_mut().welcome_pane = value;
    }
    pub(crate) fn get_welcome_pane(&self) -> WelcomePaneData {
        self.state.borrow().welcome_pane.clone()
    }
    pub(crate) fn set_mesh_import_path(&self, _value: SharedString) {}
    pub(crate) fn set_viewport_image(&self, _value: Image) {}
    pub(crate) fn set_welcome_recent_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_hovered_welcome_recent_index(&self, _value: i32) {}
    pub(crate) fn set_hovered_welcome_recent_action(&self, _value: i32) {}
    pub(crate) fn set_hierarchy_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_hovered_hierarchy_index(&self, _value: i32) {}
    pub(crate) fn set_console_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_inspector_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_details_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_activity_asset_tree_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_activity_asset_tree_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_activity_asset_content_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_activity_asset_content_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_activity_asset_references_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_activity_asset_references_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_activity_asset_used_by_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_activity_asset_used_by_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_tree_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_browser_asset_tree_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_content_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_browser_asset_content_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_references_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_browser_asset_references_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_used_by_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_browser_asset_used_by_scroll_px(&self, _value: f32) {}

    callback_methods!(pane_callbacks, on_welcome_recent_pointer_clicked, invoke_welcome_recent_pointer_clicked, welcome_recent_pointer_clicked, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_welcome_recent_pointer_moved, invoke_welcome_recent_pointer_moved, welcome_recent_pointer_moved, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_welcome_recent_pointer_scrolled, invoke_welcome_recent_pointer_scrolled, welcome_recent_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_clicked, invoke_hierarchy_pointer_clicked, hierarchy_pointer_clicked, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_moved, invoke_hierarchy_pointer_moved, hierarchy_pointer_moved, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_scrolled, invoke_hierarchy_pointer_scrolled, hierarchy_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_event, invoke_hierarchy_pointer_event, hierarchy_pointer_event, (kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_console_pointer_scrolled, invoke_console_pointer_scrolled, console_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_inspector_pointer_scrolled, invoke_inspector_pointer_scrolled, inspector_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_inspector_reference_pointer_event, invoke_inspector_reference_pointer_event, inspector_reference_pointer_event, (kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_inspector_control_changed, invoke_inspector_control_changed, inspector_control_changed, (control_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_inspector_control_clicked, invoke_inspector_control_clicked, inspector_control_clicked, (control_id: SharedString));
    callback_methods!(pane_callbacks, on_surface_control_clicked, invoke_surface_control_clicked, surface_control_clicked, (control_id: SharedString, action_id: SharedString));
    callback_methods!(pane_callbacks, on_component_showcase_control_activated, invoke_component_showcase_control_activated, component_showcase_control_activated, (control_id: SharedString, action_id: SharedString));
    callback_methods!(pane_callbacks, on_component_showcase_control_drag_delta, invoke_component_showcase_control_drag_delta, component_showcase_control_drag_delta, (control_id: SharedString, action_id: SharedString, delta: f32));
    callback_methods!(pane_callbacks, on_component_showcase_control_edited, invoke_component_showcase_control_edited, component_showcase_control_edited, (control_id: SharedString, action_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_component_showcase_control_context_requested, invoke_component_showcase_control_context_requested, component_showcase_control_context_requested, (control_id: SharedString, action_id: SharedString, x: f32, y: f32));
    callback_methods!(pane_callbacks, on_component_showcase_option_selected, invoke_component_showcase_option_selected, component_showcase_option_selected, (control_id: SharedString, action_id: SharedString, option_id: SharedString));
    callback_methods!(pane_callbacks, on_mesh_import_path_edited, invoke_mesh_import_path_edited, mesh_import_path_edited, (value: SharedString));
    callback_methods!(pane_callbacks, on_asset_control_changed, invoke_asset_control_changed, asset_control_changed, (source: SharedString, control_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_asset_control_clicked, invoke_asset_control_clicked, asset_control_clicked, (source: SharedString, control_id: SharedString));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_clicked, invoke_asset_tree_pointer_clicked, asset_tree_pointer_clicked, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_moved, invoke_asset_tree_pointer_moved, asset_tree_pointer_moved, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_scrolled, invoke_asset_tree_pointer_scrolled, asset_tree_pointer_scrolled, (surface_mode: SharedString, x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_clicked, invoke_asset_content_pointer_clicked, asset_content_pointer_clicked, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_event, invoke_asset_content_pointer_event, asset_content_pointer_event, (surface_mode: SharedString, kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_moved, invoke_asset_content_pointer_moved, asset_content_pointer_moved, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_scrolled, invoke_asset_content_pointer_scrolled, asset_content_pointer_scrolled, (surface_mode: SharedString, x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_clicked, invoke_asset_reference_pointer_clicked, asset_reference_pointer_clicked, (surface_mode: SharedString, list_kind: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_event, invoke_asset_reference_pointer_event, asset_reference_pointer_event, (surface_mode: SharedString, list_kind: SharedString, kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_moved, invoke_asset_reference_pointer_moved, asset_reference_pointer_moved, (surface_mode: SharedString, list_kind: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_scrolled, invoke_asset_reference_pointer_scrolled, asset_reference_pointer_scrolled, (surface_mode: SharedString, list_kind: SharedString, x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_browser_asset_details_pointer_scrolled, invoke_browser_asset_details_pointer_scrolled, browser_asset_details_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_welcome_control_changed, invoke_welcome_control_changed, welcome_control_changed, (control_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_welcome_control_clicked, invoke_welcome_control_clicked, welcome_control_clicked, (control_id: SharedString));
    callback_methods!(pane_callbacks, on_viewport_pointer_event, invoke_viewport_pointer_event, viewport_pointer_event, (kind: i32, button: i32, x: f32, y: f32, delta: f32));
    callback_methods!(pane_callbacks, on_viewport_toolbar_pointer_clicked, invoke_viewport_toolbar_pointer_clicked, viewport_toolbar_pointer_clicked, (surface_key: SharedString, control_id: SharedString, control_x: f32, control_y: f32, control_width: f32, control_height: f32, point_x: f32, point_y: f32));
    callback_methods!(pane_callbacks, on_ui_asset_action, invoke_ui_asset_action, ui_asset_action, (instance_id: SharedString, action_id: SharedString));
    callback_methods!(pane_callbacks, on_ui_asset_detail_event, invoke_ui_asset_detail_event, ui_asset_detail_event, (instance_id: SharedString, detail_id: SharedString, action_id: SharedString, item_index: i32, primary: SharedString, secondary: SharedString));
    callback_methods!(pane_callbacks, on_ui_asset_collection_event, invoke_ui_asset_collection_event, ui_asset_collection_event, (instance_id: SharedString, collection_id: SharedString, event_kind: SharedString, item_index: i32));
}
