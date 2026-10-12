//! 路由器把布局同步、表面维护、事件分发和可见空间产品采用分开，所有路径共用一份最终解析状态。

#[path = "build_dispatcher.rs"]
mod build_dispatcher;
#[path = "frame_from_points.rs"]
mod frame_from_points;
#[path = "rebuild_surface.rs"]
mod rebuild_surface;
#[path = "../../../pointer/overlay_router/selectable_owners_in_rect.rs"]
mod selectable_owners_in_rect;
#[path = "router.rs"]
mod viewport_overlay_pointer_router;
#[path = "clone.rs"]
mod viewport_overlay_pointer_router_clone;
#[path = "debug.rs"]
mod viewport_overlay_pointer_router_debug;
#[path = "event.rs"]
mod viewport_overlay_pointer_router_event;
#[path = "new.rs"]
mod viewport_overlay_pointer_router_new;
#[path = "sync.rs"]
mod viewport_overlay_pointer_router_sync;
#[path = "visible_spatial_query.rs"]
mod viewport_overlay_pointer_router_visible_spatial_query;

pub(in crate::scene::viewport::pointer) use frame_from_points::frame_from_points;
pub(crate) use viewport_overlay_pointer_router::ViewportOverlayPointerRouter;
