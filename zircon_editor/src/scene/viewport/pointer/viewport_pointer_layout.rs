use zircon_framework::render::{
    HandleOverlayExtract, SceneGizmoOverlayExtract, ViewportCameraSnapshot,
};
use zircon_math::UVec2;

use super::viewport_renderable_pick_candidate::ViewportRenderablePickCandidate;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportPointerLayout {
    pub viewport: UVec2,
    pub camera: ViewportCameraSnapshot,
    pub handles: Vec<HandleOverlayExtract>,
    pub scene_gizmos: Vec<SceneGizmoOverlayExtract>,
    pub renderables: Vec<ViewportRenderablePickCandidate>,
}

impl Default for ViewportPointerLayout {
    fn default() -> Self {
        Self {
            viewport: UVec2::new(1, 1),
            camera: ViewportCameraSnapshot::default(),
            handles: Vec::new(),
            scene_gizmos: Vec::new(),
            renderables: Vec::new(),
        }
    }
}
