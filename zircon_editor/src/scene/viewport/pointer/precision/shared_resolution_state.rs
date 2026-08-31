use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use zircon_runtime::core::framework::picking::PickingDebugFeed;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::{PrecisionCandidate, RendererVisibleSpatialPickSource};
use crate::scene::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;

#[derive(Default)]
pub(in crate::scene::viewport::pointer) struct SharedResolutionState {
    pub(in crate::scene::viewport::pointer) candidates: BTreeMap<UiNodeId, PrecisionCandidate>,
    pub(in crate::scene::viewport::pointer) renderer_visible_spatial_pick_source:
        Option<RendererVisibleSpatialPickSource>,
    pub(in crate::scene::viewport::pointer) last_route: Option<ViewportPointerRoute>,
    pub(in crate::scene::viewport::pointer) last_debug_feed: Option<PickingDebugFeed>,
}

/// Keeps the pointer router usable after an isolated callback has poisoned its shared state.
pub(in crate::scene::viewport::pointer) fn lock_shared_resolution_state(
    state: &Mutex<SharedResolutionState>,
) -> MutexGuard<'_, SharedResolutionState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
