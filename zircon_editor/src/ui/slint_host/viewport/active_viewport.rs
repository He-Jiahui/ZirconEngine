use crate::scene::viewport::RenderViewportHandle;
use zircon_runtime::core::math::UVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActiveViewport {
    pub(super) handle: RenderViewportHandle,
    pub(super) size: UVec2,
}
