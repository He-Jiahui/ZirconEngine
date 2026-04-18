use zircon_math::Vec2;
use zircon_scene::{Scene, SceneViewportSettings, ViewportCameraSnapshot};

#[derive(Clone, Debug)]
pub(crate) struct HandlePickContext<'a> {
    pub(crate) scene: &'a Scene,
    pub(crate) selected: Option<u64>,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) camera: &'a ViewportCameraSnapshot,
    pub(crate) cursor: Vec2,
}
