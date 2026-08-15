use std::collections::VecDeque;

use thiserror::Error;

use crate::asset::artifact::{
    write_ibl_bake_artifact_runtime_dispatch_readback, IblBakeArtifactCacheStore,
    IblBakeArtifactRuntimeDispatchError, IblBakeArtifactRuntimeDispatchReadbackReport,
    IblBakeArtifactRuntimeDispatchReport,
};
use crate::core::framework::render::{
    IblBakeArtifactDescriptor, IblBakeArtifactReadbackSections, IblBakeArtifactRequest,
};
use crate::graphics::backend::IblBakeArtifactWgpuPendingReadback;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::types::GraphicsError;
use crate::graphics::EnvironmentIblBakeReservation;

use super::ibl_bake_wgpu_readback::{
    prepare_ibl_bake_artifact_wgpu_readback_from_graph_resources,
    read_ibl_bake_artifact_wgpu_sections_from_graph_resources,
};

const MAX_PENDING_IBL_BAKE_RUNTIME_WRITEBACKS: usize = 4;

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeRuntimeGraphWritebackQueue {
    pending: VecDeque<PreparedIblBakeRuntimeGraphWriteback>,
}

impl IblBakeRuntimeGraphWritebackQueue {
    pub(in crate::graphics::scene::scene_renderer) fn prepare(
        &self,
        device: &wgpu::Device,
        store: IblBakeArtifactCacheStore,
        request: IblBakeArtifactRequest,
        reservation: EnvironmentIblBakeReservation,
        resources: &RenderGraphExecutionResources,
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
            device, descriptor, resources,
        )
        .map_err(IblBakeRuntimeGraphWritebackError::Readback)?;
        Ok(Some(PreparedIblBakeRuntimeGraphWriteback {
            store,
            request,
            dispatch,
            readback,
            _reservation: reservation,
        }))
    }

    pub(in crate::graphics::scene::scene_renderer) fn commit_submitted(
        &mut self,
        mut prepared: PreparedIblBakeRuntimeGraphWriteback,
    ) {
        prepared.readback.begin_map();
        self.pending.push_back(prepared);
    }

    pub(in crate::graphics::scene::scene_renderer) fn poll_completed(
        &mut self,
        device: &wgpu::Device,
    ) -> Result<(), IblBakeRuntimeGraphWritebackError> {
        if let Err(error) = device.poll(wgpu::PollType::Poll) {
            // Device-poll failure makes every queued readback unrecoverable. Releasing their
            // reservations lets a recovered device author a fresh IBL graph on the next frame.
            self.pending.clear();
            return Err(IblBakeRuntimeGraphWritebackError::Readback(
                GraphicsError::BufferMap(error.to_string()),
            ));
        }
        let mut index = 0;
        while index < self.pending.len() {
            let ready = match self.pending[index].readback.poll_ready() {
                Ok(ready) => ready,
                Err(error) => {
                    self.pending.remove(index);
                    return Err(IblBakeRuntimeGraphWritebackError::Readback(error));
                }
            };
            if !ready {
                index += 1;
                continue;
            }
            let Some(pending) = self.pending.remove(index) else {
                break;
            };
            let readback = pending
                .readback
                .finish()
                .map_err(IblBakeRuntimeGraphWritebackError::Readback)?;
            write_ibl_bake_artifact_runtime_dispatch_readback(
                &pending.store,
                &pending.request,
                &pending.dispatch,
                readback,
            )
            .map_err(IblBakeRuntimeGraphWritebackError::RuntimeDispatch)?;
        }
        Ok(())
    }
}

pub(in crate::graphics::scene::scene_renderer) struct PreparedIblBakeRuntimeGraphWriteback {
    store: IblBakeArtifactCacheStore,
    request: IblBakeArtifactRequest,
    dispatch: IblBakeArtifactRuntimeDispatchReport,
    readback: IblBakeArtifactWgpuPendingReadback,
    _reservation: EnvironmentIblBakeReservation,
}

impl PreparedIblBakeRuntimeGraphWriteback {
    pub(in crate::graphics::scene::scene_renderer) fn take_command_buffer(
        &mut self,
    ) -> Option<wgpu::CommandBuffer> {
        self.readback.take_command_buffer()
    }
}

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
mod tests;
