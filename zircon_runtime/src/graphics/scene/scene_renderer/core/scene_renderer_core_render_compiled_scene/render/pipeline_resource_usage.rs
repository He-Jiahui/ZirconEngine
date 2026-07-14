use crate::graphics::CompiledRenderPipeline;
use crate::render_graph::RenderGraphResourceAccessKind;

pub(super) fn pipeline_writes_resource(
    pipeline: &CompiledRenderPipeline,
    resource_name: &str,
) -> bool {
    pipeline
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .flat_map(|pass| pass.resources.iter())
        .any(|resource| {
            resource.name == resource_name
                && resource.access == RenderGraphResourceAccessKind::Write
        })
}
