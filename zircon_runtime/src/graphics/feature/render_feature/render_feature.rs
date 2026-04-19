use crate::core::framework::render::RenderFrameExtract;
use crate::render_graph::RenderGraphBuilder;

use crate::graphics::feature::RenderFeatureDescriptor;

pub trait RenderFeature: Send + Sync {
    fn descriptor(&self) -> RenderFeatureDescriptor;

    fn register_passes(
        &self,
        _graph: &mut RenderGraphBuilder,
        _extract: &RenderFrameExtract,
    ) -> Result<(), String> {
        Ok(())
    }
}
