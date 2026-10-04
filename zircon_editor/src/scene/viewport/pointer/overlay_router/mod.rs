//! 路由器把布局同步、表面维护、事件分发和可见空间产品采用分开，所有路径共用一份最终解析状态。

mod build_dispatcher;
mod frame_from_points;
mod rebuild_surface;
mod selectable_owners_in_rect;
mod viewport_overlay_pointer_router;
mod viewport_overlay_pointer_router_clone;
mod viewport_overlay_pointer_router_debug;
mod viewport_overlay_pointer_router_event;
mod viewport_overlay_pointer_router_new;
mod viewport_overlay_pointer_router_sync;
mod viewport_overlay_pointer_router_visible_spatial_query;

pub(in crate::scene::viewport::pointer) use frame_from_points::frame_from_points;
pub(crate) use viewport_overlay_pointer_router::ViewportOverlayPointerRouter;
