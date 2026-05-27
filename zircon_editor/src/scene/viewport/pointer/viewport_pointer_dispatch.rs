use zircon_runtime::core::framework::picking::{PickingDebugFeed, PointerInput};

use super::viewport_pointer_route::ViewportPointerRoute;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportPointerDispatch {
    pub route: Option<ViewportPointerRoute>,
    pub runtime_input: Option<PointerInput>,
    pub picking_debug_feed: Option<PickingDebugFeed>,
}
