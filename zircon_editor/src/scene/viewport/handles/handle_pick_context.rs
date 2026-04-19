use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime::core::math::Vec2;
use zircon_runtime::scene::Scene;

#[derive(Clone, Debug)]
pub(crate) struct HandlePickContext<'a> {
    pub(crate) scene: &'a Scene,
    pub(crate) selected: Option<u64>,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) camera: &'a ViewportCameraSnapshot,
    pub(crate) cursor: Vec2,
}
