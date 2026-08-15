use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot;
use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::scene::viewport::pointer::{
    precision::SharedResolutionState, viewport_pointer_layout::ViewportPointerLayout,
    viewport_renderable_pick_candidate::ViewportRenderablePickCandidate,
};
use crate::scene::viewport::ViewportInteractionExtract;

pub(crate) struct ViewportOverlayPointerRouter {
    pub(in crate::scene::viewport::pointer) layout: ViewportPointerLayout,
    pub(in crate::scene::viewport::pointer) surface: UiSurface,
    pub(in crate::scene::viewport::pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::scene::viewport::pointer) shared: Arc<Mutex<SharedResolutionState>>,
    pub(super) interaction_extract: Option<Arc<ViewportInteractionExtract>>,
    pub(super) renderable_candidates: Arc<[ViewportRenderablePickCandidate]>,
    pub(super) scene_world_generation: Option<u64>,
    pub(super) renderer_visible_spatial_snapshot: Option<RenderVisibleSpatialQuerySnapshot>,
}
