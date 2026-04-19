use crate::core::framework::render::{RenderFrameExtract, RenderSceneSnapshot};
use crate::core::math::UVec2;
use crate::ui::surface::UiRenderExtract;

use super::hybrid_gi_prepare::HybridGiPrepareFrame;
use super::hybrid_gi_resolve_runtime::HybridGiResolveRuntime;
use super::virtual_geometry_prepare::VirtualGeometryPrepareFrame;

#[derive(Clone, Debug)]
pub struct EditorOrRuntimeFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub ui: Option<UiRenderExtract>,
    pub(crate) hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    pub(crate) hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
    pub(crate) virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
}
