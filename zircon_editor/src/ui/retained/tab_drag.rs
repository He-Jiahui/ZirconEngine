#[cfg(any(test, feature = "integration-contracts"))]
#[path = "tab_drag/bridge.rs"]
mod bridge;
#[path = "tab_drag/drop_resolution.rs"]
mod drop_resolution;
#[path = "tab_drag/group.rs"]
mod group;
#[path = "tab_drag/host_resolution.rs"]
mod host_resolution;
#[path = "tab_drag/resolved_drop.rs"]
mod resolved_drop;
#[path = "tab_drag/route_resolution.rs"]
mod route_resolution;
#[path = "tab_drag/strip_hitbox.rs"]
mod strip_hitbox;
#[path = "tab_drag/tab_width.rs"]
mod tab_width;

pub use group::HostDragTargetGroup;
pub(crate) use group::{
    floating_window_edge_group_key, floating_window_group_key, host_shell_pointer_route_group_key,
    host_shell_pointer_route_matches_group_key,
};
pub(crate) use resolved_drop::{ResolvedHostTabDropRoute, ResolvedHostTabDropTarget};
pub(crate) use route_resolution::resolve_host_tab_drop_route_with_workbench_layout_frames;

#[cfg(feature = "integration-contracts")]
pub use bridge::resolve_host_drag_target_group_with_workbench_shell_geometry;
#[cfg(test)]
pub(crate) use bridge::{
    resolve_host_drag_target_group_with_root_frames,
    resolve_host_drag_target_group_with_workbench_layout_frames,
};
#[cfg(test)]
pub(crate) use drop_resolution::{
    resolve_tab_drop_with_root_frames, resolve_tab_drop_with_workbench_layout_frames,
};
#[cfg(test)]
pub(crate) use group::document_edge_group_key;
#[cfg(test)]
pub(crate) use host_resolution::{drop_host_for_group, drop_host_for_tab};
#[cfg(test)]
pub(crate) use resolved_drop::ResolvedTabDrop;
#[cfg(test)]
pub(crate) use route_resolution::resolve_host_tab_drop_route;
#[cfg(test)]
pub(crate) use route_resolution::resolve_host_tab_drop_route_with_root_frames;
#[cfg(test)]
pub(crate) use tab_width::{estimate_dock_tab_width, estimate_document_tab_width};
