use zircon_framework::render::ViewportCameraSnapshot;
use zircon_math::{UVec2, Vec2};

#[derive(Clone, Debug)]
pub(crate) struct HandleDragContext<'a> {
    pub(crate) camera: &'a ViewportCameraSnapshot,
    pub(crate) viewport: UVec2,
    pub(crate) current_cursor: Vec2,
}
