use zircon_runtime::core::framework::picking::PickingDebugFeed;

use super::viewport_pointer_route::ViewportPointerRoute;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportPointerDispatch {
    pub route: Option<ViewportPointerRoute>,
    pub picking_debug_feed: Option<PickingDebugFeed>,
}
