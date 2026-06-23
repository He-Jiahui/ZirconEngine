use crate::ui::retained_host::primitives::SharedString;

use super::super::super::data::WorkbenchContextMenuRequestData;
use super::types::{
    Callback1, Callback2, Callback3, Callback4, Callback5, Callback6, Callback7, Callback8,
};

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
        Option<Callback5<SharedString, f32, f32, f32, f32>>,
    pub(in crate::ui::retained_host::host_contract) ui_asset_action:
        Option<Callback2<SharedString, SharedString>>,
    pub(in crate::ui::retained_host::host_contract) ui_asset_detail_event: Option<
        Callback6<SharedString, SharedString, SharedString, i32, SharedString, SharedString>,
    >,
    pub(in crate::ui::retained_host::host_contract) ui_asset_collection_event:
        Option<Callback4<SharedString, SharedString, SharedString, i32>>,
}
