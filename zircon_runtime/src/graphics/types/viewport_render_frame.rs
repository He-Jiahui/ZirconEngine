use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryDebugSnapshot,
};
use crate::core::math::UVec2;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

#[derive(Clone, Debug)]
pub struct ViewportRenderFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub ui: Option<UiRenderExtract>,
    pub(crate) virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
}
