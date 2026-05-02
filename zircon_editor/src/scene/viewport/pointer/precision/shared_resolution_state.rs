use std::collections::BTreeMap;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::PrecisionCandidate;
use crate::scene::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;

#[derive(Default)]
pub(in crate::scene::viewport::pointer) struct SharedResolutionState {
    pub(in crate::scene::viewport::pointer) candidates: BTreeMap<UiNodeId, PrecisionCandidate>,
    pub(in crate::scene::viewport::pointer) last_route: Option<ViewportPointerRoute>,
}
