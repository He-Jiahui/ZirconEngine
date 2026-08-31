use std::collections::VecDeque;

use thiserror::Error;

#[cfg(test)]
use crate::asset::artifact::IblBakeArtifactRuntimeDispatchReadbackReport;
use crate::asset::artifact::{
    write_ibl_bake_artifact_runtime_dispatch_readback, IblBakeArtifactCacheStore,
    IblBakeArtifactRuntimeDispatchError, IblBakeArtifactRuntimeDispatchReport,
};
#[cfg(test)]
use crate::core::framework::render::IblBakeArtifactReadbackSections;
use crate::core::framework::render::{IblBakeArtifactDescriptor, IblBakeArtifactRequest};
use crate::graphics::backend::IblBakeArtifactWgpuPendingReadback;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::types::GraphicsError;
use crate::graphics::EnvironmentIblBakeReservation;
use crate::render_graph::CompiledRenderGraph;

use super::environment_capture_gpu_target::EnvironmentCaptureGpuTarget;
#[cfg(test)]
use super::ibl_bake_wgpu_readback::read_ibl_bake_artifact_wgpu_sections_from_graph_resources;
use super::ibl_bake_wgpu_readback::{
    prepare_ibl_bake_artifact_wgpu_readback_from_capture_target,
    prepare_ibl_bake_artifact_wgpu_readback_from_graph_resources,
};

const MAX_PENDING_IBL_BAKE_RUNTIME_WRITEBACKS: usize = 4;

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeRuntimeGraphWritebackQueue {
    pending: VecDeque<PreparedIblBakeRuntimeGraphWriteback>,
}

impl IblBakeRuntimeGraphWritebackQueue {
    pub(in crate::graphics::scene::scene_renderer) fn prepare(
        &self,
        backend: &RenderBackend,
        store: IblBakeArtifactCacheStore,
        request: IblBakeArtifactRequest,
        reservation: EnvironmentIblBakeReservation,
        resources: &RenderGraphExecutionResources,
        graph: &CompiledRenderGraph,
    ) -> Result<Option<PreparedIblBakeRuntimeGraphWriteback>, IblBakeRuntimeGraphWritebackError>
    {
        if self
            .pending
            .iter()
            .any(|pending| pending.request == request)
            || self.pending.len() >= MAX_PENDING_IBL_BAKE_RUNTIME_WRITEBACKS
        {
            return Ok(None);
        }
        let dispatch = crate::asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch(
            &store,
            &request,
            &[],
        )
        .map_err(IblBakeRuntimeGraphWritebackError::RuntimeDispatch)?;
        if !dispatch.requires_runtime_compute() {
            return Ok(None);
        }
        let descriptor = current_descriptor_for_request(&request);
        let readback = prepare_ibl_bake_artifact_wgpu_readback_from_graph_resources(
            backend, descriptor, resources, graph,
        )
        .map_err(IblBakeRuntimeGraphWritebackError::Readback)?;
        Ok(Some(PreparedIblBakeRuntimeGraphWriteback {
            store,
            request,
            dispatch,
            readback,
            _reservation: Some(reservation),
            allow_readback_failure: false,
        }))
    }

    pub(in crate::graphics::scene::scene_renderer) fn prepare_from_capture_target(
        &self,
        backend: &RenderBackend,
        store: IblBakeArtifactCacheStore,
        request: IblBakeArtifactRequest,
        target: &EnvironmentCaptureGpuTarget,
    ) -> Result<Option<PreparedIblBakeRuntimeGraphWriteback>, IblBakeRuntimeGraphWritebackError>
    {
        if self
            .pending
            .iter()
            .any(|pending| pending.request == request)
            || self.pending.len() >= MAX_PENDING_IBL_BAKE_RUNTIME_WRITEBACKS
        {
            return Ok(None);
        }
        let dispatch = crate::asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch(
            &store,
            &request,
            &[],
        )
        .map_err(IblBakeRuntimeGraphWritebackError::RuntimeDispatch)?;
        if !dispatch.requires_runtime_compute() {
            return Ok(None);
        }
        let descriptor = current_descriptor_for_request(&request);
        let readback = prepare_ibl_bake_artifact_wgpu_readback_from_capture_target(
            backend, descriptor, target,
        )
        .map_err(IblBakeRuntimeGraphWritebackError::Readback)?;
        Ok(Some(PreparedIblBakeRuntimeGraphWriteback {
            store,
            request,
            dispatch,
            readback,
            _reservation: None,
            allow_readback_failure: true,
        }))
    }

    pub(in crate::graphics::scene::scene_renderer) fn commit_submitted(
        &mut self,
        prepared: PreparedIblBakeRuntimeGraphWriteback,
    ) {
        self.pending.push_back(prepared);
    }

    pub(in crate::graphics::scene::scene_renderer) fn poll_completed(
        &mut self,
    ) -> Result<(), IblBakeRuntimeGraphWritebackError> {
        let mut index = 0;
        while index < self.pending.len() {
            let ready = self.pending[index].readback.poll_ready();
            if !ready {
                index += 1;
                continue;
            }
            let Some(pending) = self.pending.remove(index) else {
                break;
            };
            let readback = match pending.readback.finish() {
                Ok(readback) => readback,
                Err(_) if pending.allow_readback_failure => continue,
                Err(error) => return Err(IblBakeRuntimeGraphWritebackError::Readback(error)),
            };
            if let Err(error) = write_ibl_bake_artifact_runtime_dispatch_readback(
                &pending.store,
                &pending.request,
                &pending.dispatch,
                readback,
            ) {
                if pending.allow_readback_failure {
                    continue;
                }
                return Err(IblBakeRuntimeGraphWritebackError::RuntimeDispatch(error));
            }
        }
        Ok(())
    }
}

pub(in crate::graphics::scene::scene_renderer) struct PreparedIblBakeRuntimeGraphWriteback {
    store: IblBakeArtifactCacheStore,
    request: IblBakeArtifactRequest,
    dispatch: IblBakeArtifactRuntimeDispatchReport,
    readback: IblBakeArtifactWgpuPendingReadback,
    _reservation: Option<EnvironmentIblBakeReservation>,
    allow_readback_failure: bool,
}

#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer) fn write_ibl_bake_runtime_cache_from_graph_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    store: &IblBakeArtifactCacheStore,
    request: &IblBakeArtifactRequest,
    dispatch: &IblBakeArtifactRuntimeDispatchReport,
    resources: &RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<IblBakeArtifactRuntimeDispatchReadbackReport, IblBakeRuntimeGraphWritebackError> {
    let descriptor = current_descriptor_for_request(request);
    let readback = if dispatch.requires_runtime_compute() {
        read_ibl_bake_artifact_wgpu_sections_from_graph_resources(
            device, queue, descriptor, resources, graph,
        )
        .map_err(IblBakeRuntimeGraphWritebackError::Readback)?
    } else {
        IblBakeArtifactReadbackSections::new(descriptor)
    };

    write_ibl_bake_artifact_runtime_dispatch_readback(store, request, dispatch, readback)
        .map_err(IblBakeRuntimeGraphWritebackError::RuntimeDispatch)
}

fn current_descriptor_for_request(request: &IblBakeArtifactRequest) -> IblBakeArtifactDescriptor {
    IblBakeArtifactDescriptor::current_for_runtime_cache_request(request)
}

#[derive(Debug, Error)]
pub(in crate::graphics::scene::scene_renderer) enum IblBakeRuntimeGraphWritebackError {
    #[error("read IBL bake graph outputs before transient release: {0}")]
    Readback(GraphicsError),
    #[error("write IBL bake runtime dispatch readback: {0}")]
    RuntimeDispatch(IblBakeArtifactRuntimeDispatchError),
}

#[cfg(test)]
mod source_contract_tests {
    #[test]
    fn production_writeback_is_cpu_only_after_the_backend_completion_poll() {
        let source = include_str!("ibl_bake_runtime_writeback.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(in crate::graphics::scene::scene_renderer) fn write_ibl")
            .map(|(production, _)| production)
            .expect("runtime writeback must retain a test-only synchronous helper boundary");

        assert!(production.contains("readback.poll_ready()"));
        assert!(!production.contains("wgpu::Buffer"));
        assert!(!production.contains("map_async("));
        assert!(!production.contains("device.poll("));
        assert!(!production.contains("queue.submit("));
        assert!(!production.contains("take_command_buffer("));
    }

    #[test]
    fn capture_writeback_reuses_bounded_poll_owner_without_graph_resource_access() {
        let source = include_str!("ibl_bake_runtime_writeback.rs");
        let capture = source
            .split_once("prepare_from_capture_target(")
            .and_then(|(_, tail)| {
                tail.split_once(
                    "pub(in crate::graphics::scene::scene_renderer) fn commit_submitted",
                )
            })
            .map(|(capture, _)| capture)
            .expect("capture writeback preparation must remain an explicit owner");

        assert!(capture.contains("prepare_ibl_bake_artifact_wgpu_readback_from_capture_target"));
        assert!(capture.contains("self.pending.len() >= MAX_PENDING_IBL_BAKE_RUNTIME_WRITEBACKS"));
        assert!(capture.contains("allow_readback_failure: true"));
        assert!(!capture.contains("RenderGraphExecutionResources"));
        assert!(!capture.contains("owned_texture("));

        let completion = source
            .split_once("pub(in crate::graphics::scene::scene_renderer) fn poll_completed")
            .map(|(_, tail)| tail)
            .expect("runtime writeback must retain one bounded completion owner");
        assert!(completion.contains("Err(_) if pending.allow_readback_failure => continue"));
        assert!(completion.contains("if pending.allow_readback_failure"));
    }
}

#[cfg(test)]
mod tests;
