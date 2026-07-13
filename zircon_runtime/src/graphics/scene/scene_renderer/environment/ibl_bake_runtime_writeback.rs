use thiserror::Error;

use crate::asset::artifact::{
    write_ibl_bake_artifact_runtime_dispatch_readback, IblBakeArtifactCacheStore,
    IblBakeArtifactRuntimeDispatchError, IblBakeArtifactRuntimeDispatchReadbackReport,
    IblBakeArtifactRuntimeDispatchReport,
};
use crate::core::framework::render::{
    IblBakeArtifactDescriptor, IblBakeArtifactReadbackSections, IblBakeArtifactRequest,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::types::GraphicsError;

use super::ibl_bake_wgpu_readback::read_ibl_bake_artifact_wgpu_sections_from_graph_resources;

pub(in crate::graphics::scene::scene_renderer) fn write_ibl_bake_runtime_cache_from_graph_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    store: &IblBakeArtifactCacheStore,
    request: &IblBakeArtifactRequest,
    dispatch: &IblBakeArtifactRuntimeDispatchReport,
    resources: &RenderGraphExecutionResources,
) -> Result<IblBakeArtifactRuntimeDispatchReadbackReport, IblBakeRuntimeGraphWritebackError> {
    let descriptor = current_descriptor_for_request(request);
    let readback = if dispatch.requires_runtime_compute() {
        read_ibl_bake_artifact_wgpu_sections_from_graph_resources(
            device, queue, descriptor, resources,
        )
        .map_err(IblBakeRuntimeGraphWritebackError::Readback)?
    } else {
        IblBakeArtifactReadbackSections::new(descriptor)
    };

    write_ibl_bake_artifact_runtime_dispatch_readback(store, request, dispatch, readback)
        .map_err(IblBakeRuntimeGraphWritebackError::RuntimeDispatch)
}

fn current_descriptor_for_request(request: &IblBakeArtifactRequest) -> IblBakeArtifactDescriptor {
    IblBakeArtifactDescriptor::current_for_request(request)
}

#[derive(Debug, Error)]
pub(in crate::graphics::scene::scene_renderer) enum IblBakeRuntimeGraphWritebackError {
    #[error("read IBL bake graph outputs before transient release: {0}")]
    Readback(GraphicsError),
    #[error("write IBL bake runtime dispatch readback: {0}")]
    RuntimeDispatch(IblBakeArtifactRuntimeDispatchError),
}

#[cfg(test)]
mod tests;
