use zircon_scene::{Scene, SceneViewportSettings, ViewportCameraSnapshot};

#[derive(Clone, Debug)]
pub(crate) struct HandleBuildContext<'a> {
    pub(crate) scene: &'a Scene,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) camera: &'a ViewportCameraSnapshot,
}
