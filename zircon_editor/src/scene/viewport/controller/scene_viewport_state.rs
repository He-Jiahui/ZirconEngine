use crate::scene::modes::{SceneModeRegistry, SceneModeStack};
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::ViewportState;
use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime::core::framework::camera_controller::OrbitCameraController;
use zircon_runtime_interface::math::Vec3;

use super::{viewport_drag_session::ViewportDragSession, viewport_hover_state::ViewportHoverState};

#[derive(Debug)]
pub(crate) struct SceneViewportState {
    pub(crate) settings: SceneViewportSettings,
    pub(crate) selection: SelectionModel,
    pub(crate) scene_mode_registry: SceneModeRegistry,
    pub(crate) scene_modes: SceneModeStack,
    pub(crate) viewport: ViewportState,
    pub(crate) camera: Option<ViewportCameraSnapshot>,
    pub(crate) orbit_target: Vec3,
    pub(crate) orbit_controller: OrbitCameraController,
    pub(crate) hover: ViewportHoverState,
    pub(crate) drag: Option<ViewportDragSession>,
}
