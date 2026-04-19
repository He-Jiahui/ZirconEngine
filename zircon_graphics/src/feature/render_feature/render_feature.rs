use zircon_framework::render::RenderFrameExtract;
use zircon_render_graph::RenderGraphBuilder;

use crate::feature::RenderFeatureDescriptor;

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
