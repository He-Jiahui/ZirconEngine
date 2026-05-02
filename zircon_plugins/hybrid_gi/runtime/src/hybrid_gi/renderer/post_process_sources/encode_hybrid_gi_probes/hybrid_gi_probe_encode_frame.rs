use crate::hybrid_gi::types::{
    HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
};
use zircon_runtime::core::framework::render::RenderFrameExtract;
use zircon_runtime::core::math::UVec2;

#[derive(Clone, Debug, PartialEq)]
// Plugin-local probe encode input seam; concrete HGI frame state stays out of the neutral runtime frame.
pub(in crate::hybrid_gi::renderer::post_process_sources::encode_hybrid_gi_probes) struct HybridGiProbeEncodeFrame
{
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    pub hybrid_gi_scene_prepare: Option<HybridGiScenePrepareFrame>,
    pub hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
}

impl HybridGiProbeEncodeFrame {
    pub(in crate::hybrid_gi::renderer::post_process_sources::encode_hybrid_gi_probes) fn from_extract(
        extract: RenderFrameExtract,
        viewport_size: UVec2,
    ) -> Self {
        Self {
            extract,
            viewport_size,
            hybrid_gi_prepare: None,
            hybrid_gi_scene_prepare: None,
            hybrid_gi_resolve_runtime: None,
        }
    }

    pub(in crate::hybrid_gi::renderer::post_process_sources::encode_hybrid_gi_probes) fn with_hybrid_gi_prepare(
        mut self,
        hybrid_gi_prepare: Option<HybridGiPrepareFrame>,
    ) -> Self {
        self.hybrid_gi_prepare = hybrid_gi_prepare;
        self
    }

    pub(in crate::hybrid_gi::renderer::post_process_sources::encode_hybrid_gi_probes) fn with_hybrid_gi_scene_prepare(
        mut self,
        hybrid_gi_scene_prepare: Option<HybridGiScenePrepareFrame>,
    ) -> Self {
        self.hybrid_gi_scene_prepare = hybrid_gi_scene_prepare;
        self
    }

    pub(in crate::hybrid_gi::renderer::post_process_sources::encode_hybrid_gi_probes) fn with_hybrid_gi_resolve_runtime(
        mut self,
        hybrid_gi_resolve_runtime: Option<HybridGiResolveRuntime>,
    ) -> Self {
        self.hybrid_gi_resolve_runtime = hybrid_gi_resolve_runtime;
        self
    }
}
