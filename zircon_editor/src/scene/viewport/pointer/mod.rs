//! 视口指针模块借 UI 表面粗筛并交 Runtime 统一解析，编辑器保留几何呈现与 owner 映射，不另建输入权威。

#[path = "candidates/mod.rs"]
mod candidates;
#[path = "constants.rs"]
mod constants;
#[path = "local_handle_route.rs"]
mod local_handle_route;
#[path = "overlay_router/mod.rs"]
mod overlay_router;
#[path = "precision/mod.rs"]
mod precision;
#[path = "runtime_picking_adapter.rs"]
mod runtime_picking_adapter;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
#[path = "viewport_pointer_dispatch.rs"]
mod viewport_pointer_dispatch;
#[path = "viewport_pointer_layout.rs"]
mod viewport_pointer_layout;
#[path = "viewport_pointer_route.rs"]
mod viewport_pointer_route;
#[path = "viewport_renderable_pick_candidate.rs"]
mod viewport_renderable_pick_candidate;

pub(in crate::scene::viewport) use candidates::projected_ring_segments;
pub(in crate::scene::viewport) use local_handle_route::local_handle_route;
pub(crate) use overlay_router::ViewportOverlayPointerRouter;
pub(in crate::scene::viewport) use viewport_pointer_dispatch::ViewportPointerDispatch;
#[cfg(test)]
pub(crate) use viewport_pointer_layout::ViewportPointerLayout;
pub(crate) use viewport_pointer_route::ViewportPointerRoute;
#[cfg(test)]
pub(crate) use viewport_renderable_pick_candidate::ViewportRenderablePickCandidate;
