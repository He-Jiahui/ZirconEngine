use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime_interface::math::Transform;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HandleSelection {
    pub(crate) entity: u64,
    pub(crate) transform: Transform,
}

#[derive(Clone, Debug)]
pub(crate) struct HandleBuildContext<'a> {
    pub(crate) selected: Option<HandleSelection>,
    pub(crate) settings: &'a SceneViewportSettings,
    pub(crate) camera: &'a ViewportCameraSnapshot,
}
