use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec2;

#[derive(Clone, Debug)]
pub(crate) struct HandlePickContext<'a> {
    pub(crate) scene: &'a Scene,
    pub(crate) selected: Option<u64>,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) camera: &'a ViewportCameraSnapshot,
    pub(crate) cursor: Vec2,
}
