use super::scroll_surface_pointer_route::ScrollSurfacePointerRoute;
use super::scroll_surface_pointer_state::ScrollSurfacePointerState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollSurfacePointerDispatch {
    pub(crate) route: Option<ScrollSurfacePointerRoute>,
    pub(crate) state: ScrollSurfacePointerState,
    pub(crate) changed: bool,
}
