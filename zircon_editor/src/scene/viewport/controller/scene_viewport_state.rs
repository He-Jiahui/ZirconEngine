use crate::ViewportState;
use zircon_math::Vec3;
use zircon_scene::{SceneViewportSettings, ViewportCameraSnapshot};

use super::{viewport_drag_session::ViewportDragSession, viewport_hover_state::ViewportHoverState};

#[derive(Clone, Debug)]
pub(crate) struct SceneViewportState {
    pub(crate) settings: SceneViewportSettings,
    pub(crate) selected: Option<u64>,
    pub(crate) viewport: ViewportState,
    pub(crate) camera: Option<ViewportCameraSnapshot>,
    pub(crate) orbit_target: Vec3,
    pub(crate) hover: ViewportHoverState,
    pub(crate) drag: Option<ViewportDragSession>,
}
