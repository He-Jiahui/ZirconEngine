use crate::graphics::CompiledRenderPipeline;

pub(super) fn pipeline_writes_resource(
    pipeline: &CompiledRenderPipeline,
    resource_name: &str,
) -> bool {
    pipeline.writes_resource(resource_name)
}
