use crate::scene::viewport::ViewportCameraSnapshot;
use zircon_runtime_interface::math::{UVec2, Vec2};

#[derive(Clone, Debug)]
pub(crate) struct HandleDragContext<'a> {
    pub(crate) camera: &'a ViewportCameraSnapshot,
    pub(crate) viewport: UVec2,
    pub(crate) current_cursor: Vec2,
}
