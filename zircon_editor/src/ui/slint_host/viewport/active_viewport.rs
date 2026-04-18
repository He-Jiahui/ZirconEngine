use zircon_math::UVec2;
use zircon_framework::render::RenderViewportHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActiveViewport {
    pub(super) handle: RenderViewportHandle,
    pub(super) size: UVec2,
}
