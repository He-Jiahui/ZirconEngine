use crate::scene::viewport::{
    SceneViewportSettings, SceneViewportSnapSteps, ViewportCameraSnapshot,
};
use zircon_runtime_interface::math::Vec2;

use super::handle_build_context::HandleSelection;

#[derive(Clone, Debug)]
pub(crate) struct HandlePickContext<'a> {
    pub(crate) selected: Option<HandleSelection>,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) snap_steps: SceneViewportSnapSteps,
    pub(crate) camera: &'a ViewportCameraSnapshot,
    pub(crate) cursor: Vec2,
}
