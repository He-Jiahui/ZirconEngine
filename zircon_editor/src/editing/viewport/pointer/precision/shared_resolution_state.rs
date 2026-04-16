use std::collections::BTreeMap;

use zircon_ui::UiNodeId;

use super::PrecisionCandidate;
use crate::editing::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;

#[derive(Default)]
pub(in crate::editing::viewport::pointer) struct SharedResolutionState {
    pub(in crate::editing::viewport::pointer) candidates: BTreeMap<UiNodeId, PrecisionCandidate>,
    pub(in crate::editing::viewport::pointer) last_route: Option<ViewportPointerRoute>,
}
