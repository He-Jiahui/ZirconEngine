use zircon_math::UVec2;
use zircon_scene::{RenderFrameExtract, RenderSceneSnapshot};

use super::hybrid_gi_prepare::HybridGiPrepareFrame;
use super::virtual_geometry_prepare::VirtualGeometryPrepareFrame;

#[derive(Clone, Debug)]
pub struct EditorOrRuntimeFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub(crate) hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    pub(crate) virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
}
