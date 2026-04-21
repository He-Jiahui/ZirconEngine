use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryDebugSnapshot,
};
use crate::core::math::UVec2;
use crate::ui::surface::UiRenderExtract;

use super::hybrid_gi_prepare::{HybridGiPrepareFrame, HybridGiScenePrepareFrame};
use super::hybrid_gi_resolve_runtime::HybridGiResolveRuntime;
use super::virtual_geometry_cluster_selection::VirtualGeometryClusterSelection;
use super::virtual_geometry_prepare::VirtualGeometryPrepareFrame;

#[derive(Clone, Debug)]
pub struct ViewportRenderFrame {
    pub scene: RenderSceneSnapshot,
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub ui: Option<UiRenderExtract>,
    pub(crate) hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    pub(crate) hybrid_gi_scene_prepare: Option<HybridGiScenePrepareFrame>,
    pub(crate) hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
    pub(crate) virtual_geometry_cluster_selections: Option<Vec<VirtualGeometryClusterSelection>>,
    pub(crate) virtual_geometry_prepare: Option<VirtualGeometryPrepareFrame>,
    pub(crate) virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
}
