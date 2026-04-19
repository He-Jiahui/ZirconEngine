use zircon_framework::render::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_scene::Scene;

#[derive(Clone, Debug)]
pub(crate) struct HandleBuildContext<'a> {
    pub(crate) scene: &'a Scene,
    pub(crate) selected: Option<u64>,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) camera: &'a ViewportCameraSnapshot,
}
