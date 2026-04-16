mod bridge;
mod drop_resolution;
mod group;
mod host_resolution;
mod resolved_drop;
mod route_resolution;
mod strip_hitbox;
mod tab_width;

pub use bridge::resolve_workbench_drag_target_group;
pub use group::WorkbenchDragTargetGroup;
pub(crate) use group::{
    floating_window_edge_group_key, floating_window_group_key,
    workbench_shell_pointer_route_group_key,
};
pub(crate) use resolved_drop::{ResolvedWorkbenchTabDropRoute, ResolvedWorkbenchTabDropTarget};
pub(crate) use route_resolution::resolve_workbench_tab_drop_route;

#[cfg(test)]
pub(crate) use drop_resolution::resolve_tab_drop;
#[cfg(test)]
pub(crate) use group::document_edge_group_key;
#[cfg(test)]
pub(crate) use host_resolution::{drop_host_for_group, drop_host_for_tab};
#[cfg(test)]
pub(crate) use resolved_drop::ResolvedTabDrop;
#[cfg(test)]
pub(crate) use tab_width::{estimate_dock_tab_width, estimate_document_tab_width};
