mod base_state;
mod build_host_activity_rail_pointer_layout;
mod collect_tabs;
mod constants;
mod dispatch_event;
mod global_point_for_side;
mod handle_click;
mod host_activity_rail_pointer_bridge;
mod host_activity_rail_pointer_dispatch;
mod host_activity_rail_pointer_item;
mod host_activity_rail_pointer_layout;
mod host_activity_rail_pointer_route;
mod host_activity_rail_pointer_side;
mod insert_strip;
mod new;
mod parse;
mod rebuild_surface;
mod register_handled_pointer_node;
mod root_frame;
mod strip_button_node_id;
mod sync;

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
