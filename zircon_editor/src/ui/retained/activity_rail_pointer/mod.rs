#[path = "base_state.rs"]
mod base_state;
#[path = "../../build_host_activity_rail_pointer_layout.rs"]
mod build_host_activity_rail_pointer_layout;
#[path = "collect_tabs.rs"]
mod collect_tabs;
#[path = "constants.rs"]
mod constants;
#[path = "dispatch_event.rs"]
mod dispatch_event;
#[path = "global_point_for_side.rs"]
mod global_point_for_side;
#[path = "handle_click.rs"]
mod handle_click;
#[path = "../../activity_rail_pointer/host_activity_rail_pointer_bridge.rs"]
mod host_activity_rail_pointer_bridge;
#[path = "../../host_activity_rail_pointer_dispatch.rs"]
mod host_activity_rail_pointer_dispatch;
#[path = "../../activity_rail_pointer/host_activity_rail_pointer_item.rs"]
mod host_activity_rail_pointer_item;
#[path = "../../activity_rail_pointer/host_activity_rail_pointer_layout.rs"]
mod host_activity_rail_pointer_layout;
#[path = "../../activity_rail_pointer/host_activity_rail_pointer_route.rs"]
mod host_activity_rail_pointer_route;
#[path = "../../activity_rail_pointer/host_activity_rail_pointer_side.rs"]
mod host_activity_rail_pointer_side;
#[path = "insert_strip.rs"]
mod insert_strip;
#[path = "new.rs"]
mod new;
#[path = "parse.rs"]
mod parse;
#[path = "rebuild_surface.rs"]
mod rebuild_surface;
#[path = "../../activity_rail_pointer/register_handled_pointer_node.rs"]
mod register_handled_pointer_node;
#[path = "root_frame.rs"]
mod root_frame;
#[path = "strip_button_node_id.rs"]
mod strip_button_node_id;
#[path = "surface_delta.rs"]
mod surface_delta;
#[path = "sync.rs"]
mod sync;

pub(crate) use build_host_activity_rail_pointer_layout::build_host_activity_rail_pointer_geometry_layout;
#[cfg(test)]
pub(crate) use build_host_activity_rail_pointer_layout::build_host_activity_rail_pointer_layout;
pub(crate) use build_host_activity_rail_pointer_layout::build_host_activity_rail_pointer_layout_with_workbench_layout_frames;
pub(crate) use host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
pub(crate) use host_activity_rail_pointer_dispatch::HostActivityRailPointerDispatch;
#[cfg(test)]
pub(crate) use host_activity_rail_pointer_item::HostActivityRailPointerItem;
#[cfg(test)]
pub(crate) use host_activity_rail_pointer_layout::HostActivityRailPointerLayout;
pub(crate) use host_activity_rail_pointer_route::HostActivityRailPointerRoute;
pub(crate) use host_activity_rail_pointer_side::HostActivityRailPointerSide;
