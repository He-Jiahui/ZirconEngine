use std::sync::Arc;

use crate::asset::{TextureAsset, TexturePayload};
use crate::core::framework::render::IblBakeArtifactRequest;
#[cfg(test)]
use crate::core::framework::render::PostProcessGraphResourceNames;
#[cfg(test)]
use crate::core::framework::render::{
    RenderColorGradingSettings, RenderColorLookupSettings, RenderColorLookupTextureLayout,
    RenderColorLutReadbackReport, RenderExposureReadbackReport, RenderImageDescriptor,
    RenderPostProcessEffectStackSettings, RenderSceneVelocityReadbackReport, RenderTonemapOperator,
    RenderTonemapSettings,
};
use crate::graphics::backend::RenderBackend;
#[cfg(test)]
use crate::graphics::backend::{read_buffer_f32x4, read_texture_rgba, read_texture_rgba16float_3d};
use crate::graphics::pipeline::CompiledRenderPipeline;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::{
    IblBakeRuntimeGraphWritebackQueue, PreparedIblBakeRuntimeGraphWriteback,
};
use crate::graphics::scene::scene_renderer::environment::RealtimeIblPendingSubmission;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryFrameTransaction,
};
use crate::graphics::types::GraphicsError;
#[cfg(test)]
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::EnvironmentIblBakeReservation;
use crate::render_graph::{
    CompiledRenderGraph, QueueLane, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessRange, RenderGraphResourceDesc,
    RenderGraphResourceState as GraphResourceState, RenderGraphTextureAspect,
};
#[cfg(test)]
use crate::rhi::TextureFormat;
use crate::rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, RhiGraphAccessId, RhiGraphAccessRange,
    RhiGraphExecutionAccess, RhiGraphExecutionPass, RhiGraphExecutionReceipt,
    RhiGraphExecutionTransition, RhiGraphPhysicalResourceLease, RhiGraphQueueLane,
    RhiGraphResourceAccessKind, RhiGraphResourceBounds, RhiGraphResourceId, RhiGraphResourceKind,
    RhiGraphResourceState as RhiResourceState, SubmissionTicket,
};
use zr_rhi_wgpu::{
    WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticReadbackFrame, WgpuNativeSurfaceFrameTarget,
};

use super::super::super::scene_renderer_core::SceneRendererCore;

#[path = "submit_compiled_scene_frame/hzb_readback.rs"]
mod hzb_readback;
use hzb_readback::attach_hzb_occlusion_readback_stats;

pub(super) struct CompiledSceneFrameSubmissionContext<'a> {
    pub(super) backend: &'a RenderBackend,
    pub(super) pipeline: &'a CompiledRenderPipeline,
    #[cfg(test)]
    pub(super) device: &'a wgpu::Device,
    #[cfg(test)]
    pub(super) queue: &'a wgpu::Queue,
    pub(super) command_buffers: Vec<wgpu::CommandBuffer>,
    #[cfg(test)]
    pub(super) streamer: &'a ResourceStreamer,
    #[cfg(test)]
    pub(super) frame: &'a ViewportRenderFrame,
    pub(super) graph_resources: &'a mut RenderGraphExecutionResources,
    pub(super) graph_execution_record: &'a mut RenderGraphExecutionRecord,
    pub(super) prepared_ibl_writeback: Option<PreparedIblBakeRuntimeGraphWriteback>,
    pub(super) environment_ibl_prepare_error: Option<GraphicsError>,
    pub(super) realtime_ibl_submission: Option<RealtimeIblPendingSubmission>,
    pub(super) diagnostic_frame_index: u64,
    pub(super) product_diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
    pub(super) product_diagnostic_query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
    pub(super) surface_target: Option<&'a WgpuNativeSurfaceFrameTarget>,
    pub(super) history_textures: Option<&'a mut SceneFrameHistoryTextures>,
    pub(super) history_frame_transaction: SceneHistoryFrameTransaction,
    pub(super) frame_generation: u64,
    pub(super) exposure_history_reset_prepared: bool,
}

impl SceneRendererCore {
    pub(super) fn submit_compiled_scene_frame(
        &mut self,
        ctx: CompiledSceneFrameSubmissionContext<'_>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        let CompiledSceneFrameSubmissionContext {
            backend,
            pipeline,
            #[cfg(test)]
            device,
            #[cfg(test)]
            queue,
            command_buffers,
            #[cfg(test)]
            streamer,
            #[cfg(test)]
            frame,
            graph_resources,
            graph_execution_record,
            prepared_ibl_writeback,
            environment_ibl_prepare_error,
            realtime_ibl_submission,
            diagnostic_frame_index,
            product_diagnostic_frame,
            product_diagnostic_query_frame,
            surface_target,
            history_textures,
            history_frame_transaction,
            frame_generation,
            exposure_history_reset_prepared,
        } = ctx;

        debug_assert!(!command_buffers.is_empty());
        let graph_execution_receipt =
            build_graph_execution_receipt(backend, pipeline, graph_resources, frame_generation)?;
        let submission_ticket = backend
            .submit_graphics_command_buffers_with_graph_receipt_and_frame_diagnostics_and_surface(
                command_buffers,
                graph_execution_receipt,
                product_diagnostic_frame,
                product_diagnostic_query_frame,
                surface_target,
            )?;
        let history_domains_report = if let Some(history) = history_textures {
            if exposure_history_reset_prepared {
                let committed = history.commit_exposure_history_reset();
                debug_assert!(committed);
            }
            history.commit_history_frame(history_frame_transaction, frame_generation)
        } else {
            Default::default()
        };
        graph_execution_record.set_history_domains_report(history_domains_report);
        self.mesh_pipelines
            .bind_recorded_pipeline_usage_to_submission(submission_ticket);
        self.commit_scene_environment_frame();
        self.mesh_pipelines
            .reflection_probes
            .commit_pending_uploads();
        if let Some(prepared) = prepared_ibl_writeback {
            self.ibl_bake_runtime_writebacks.commit_submitted(prepared);
        }
        if let Some(submission) = realtime_ibl_submission {
            self.realtime_ibl.complete_submission(submission, true);
        }
        #[cfg(test)]
        attach_scene_velocity_readback_stats(
            device,
            queue,
            graph_resources,
            graph_execution_record,
        );
        #[cfg(test)]
        let exposure_readback_report =
            attach_exposure_readback_stats(device, queue, graph_resources, graph_execution_record);
        #[cfg(test)]
        attach_color_lut_readback_stats(
            device,
            queue,
            streamer,
            frame,
            graph_resources,
            exposure_readback_report,
            graph_execution_record,
        );
        if let Some(hzb_occlusion_culler) = self.hzb_occlusion_culler.as_ref() {
            attach_hzb_occlusion_readback_stats(
                hzb_occlusion_culler,
                Some(diagnostic_frame_index),
                graph_execution_record,
            );
        }
        graph_resources.retire_transient_backings_after_submission(
            &mut self.transient_resource_pool,
            submission_ticket,
        );
        graph_execution_record.set_graph_submission(submission_ticket);
        self.transient_resource_pool.end_frame();
        graph_execution_record.set_resource_report(
            graph_execution_record
                .resource_report()
                .with_transient_pool_report(self.transient_resource_pool.last_frame_report()),
        );
        let environment_ibl_error = environment_ibl_prepare_error.map(|error| error.to_string());
        if environment_ibl_error.is_some() {
            return Err(GraphicsError::FrameFailedAfterSceneSubmission {
                scene_submission: submission_ticket,
                source: Box::new(GraphicsError::SceneSubmissionFinalization {
                    readback: None,
                    environment_ibl: environment_ibl_error,
                }),
            });
        }
        Ok(submission_ticket)
    }
}

/// Lowers compiler-owned access rows into the device-qualified proof retained by the
/// submission timeline.  Every non-logical access must resolve to a concrete WGPU owner and
/// descriptor; a missing view, descriptor, range bound, epoch, or generation fails the frame
/// before a command buffer is admitted.
fn build_graph_execution_receipt(
    backend: &RenderBackend,
    pipeline: &CompiledRenderPipeline,
    graph_resources: &RenderGraphExecutionResources,
    frame_generation: u64,
) -> Result<RhiGraphExecutionReceipt, GraphicsError> {
    let graph = pipeline.graph();
    let execution_passes = pipeline
        .execution_passes_in_graph_order()
        .filter(|execution_pass| {
            graph
                .passes()
                .get(execution_pass.graph_pass_index)
                .is_some_and(|pass| !pass.culled)
        })
        .collect::<Vec<_>>();
    let first_pass = execution_passes.first().ok_or_else(|| {
        GraphicsError::WgpuValidation(
            "compiled render graph has no live execution pass for a GPU receipt".to_owned(),
        )
    })?;
    let graph_pass = graph
        .passes()
        .get(first_pass.graph_pass_index)
        .ok_or_else(|| {
            GraphicsError::WgpuValidation(format!(
                "compiled render graph execution pass index {} is missing",
                first_pass.graph_pass_index
            ))
        })?;
    let graph_generation = graph_pass.id.generation();

    for execution_pass in &execution_passes {
        let pass = graph
            .passes()
            .get(execution_pass.graph_pass_index)
            .ok_or_else(|| {
                GraphicsError::WgpuValidation(format!(
                    "compiled render graph execution pass index {} is missing",
                    execution_pass.graph_pass_index
                ))
            })?;
        if pass.id.generation() != graph_generation {
            return Err(GraphicsError::WgpuValidation(
                "compiled render graph mixes pass generations".to_owned(),
            ));
        }
    }

    let profile = backend.device_profile();
    let device_id = profile.device_id();
    let generation = profile.generation();
    let (epoch_device_id, epoch_generation) = graph_resources
        .device_epoch()
        .ok_or_else(|| {
            GraphicsError::WgpuValidation("graph execution has no device epoch".to_owned())
        })?
        .raw_parts();
    if epoch_device_id != device_id.raw() || epoch_generation != generation.raw() {
        return Err(GraphicsError::WgpuValidation(format!(
            "graph execution device epoch ({epoch_device_id}:{epoch_generation}) is stale for backend ({:?}:{:?})",
            device_id,
            generation
        )));
    }

    let mut passes = Vec::with_capacity(execution_passes.len());
    let mut leases = Vec::new();
    for execution_pass in execution_passes {
        let pass = graph
            .passes()
            .get(execution_pass.graph_pass_index)
            .expect("live execution pass was checked above");
        let queue = rhi_queue_lane(pass.queue);
        let mut accesses = Vec::with_capacity(execution_pass.device_access_bindings.len());
        for (ordinal, binding) in execution_pass.device_access_bindings.iter().enumerate() {
            let access_id = rhi_access_id(binding.key.access_id);
            if access_id.access_ordinal() != ordinal {
                return Err(GraphicsError::WgpuValidation(format!(
                    "compiled graph pass `{}` access ordinal {} does not match compiler binding",
                    pass.name, ordinal
                )));
            }
            let declaration = graph
                .resource_declaration(binding.key.resource)
                .ok_or_else(|| {
                    GraphicsError::WgpuValidation(format!(
                        "compiled graph access {:?} has no resource declaration",
                        binding.key.access_id
                    ))
                })?;
            let resource = rhi_resource_id(binding.key.resource, graph_generation)?;
            let range = rhi_access_range(graph_resources, declaration, binding.key.range)?;
            let allocation_id = binding
                .physical_allocation
                .map(|allocation| allocation.allocation_id().index() as u64);
            let has_physical_binding = range != RhiGraphAccessRange::UnresolvedExternal;
            let access = RhiGraphExecutionAccess::new(
                access_id,
                resource,
                binding.key.version.ordinal(),
                rhi_access_kind(binding.key.access),
                range,
                rhi_resource_state(GraphResourceState::from(binding.key.intent)),
                queue,
                allocation_id,
                has_physical_binding,
            );
            if has_physical_binding {
                let lease = physical_lease(
                    graph_resources,
                    declaration,
                    access_id,
                    resource,
                    allocation_id,
                    range,
                    device_id,
                    generation,
                )?;
                leases.push(lease);
            }
            accesses.push(access);
        }

        let transitions = execution_pass
            .device_transitions_before
            .iter()
            .map(|transition| {
                let resource = rhi_resource_id(transition.resource, graph_generation)?;
                let declaration =
                    graph
                        .resource_declaration(transition.resource)
                        .ok_or_else(|| {
                            GraphicsError::WgpuValidation(format!(
                                "compiled graph transition resource {:?} has no declaration",
                                transition.resource
                            ))
                        })?;
                let range = rhi_access_range(graph_resources, declaration, transition.range)?;
                Ok(RhiGraphExecutionTransition::new(
                    resource,
                    range,
                    rhi_access_id(transition.from_access),
                    rhi_access_id(transition.to_access),
                    rhi_resource_state(transition.from_state),
                    rhi_resource_state(transition.to_state),
                    rhi_queue_lane(transition.from_queue),
                    rhi_queue_lane(transition.to_queue),
                ))
            })
            .collect::<Result<Vec<_>, GraphicsError>>()?;
        passes.push(RhiGraphExecutionPass::new(
            execution_pass.graph_pass_index,
            pass.id.index(),
            graph_generation,
            queue,
            accesses,
            transitions,
        ));
    }

    RhiGraphExecutionReceipt::new(
        device_id,
        generation,
        frame_generation,
        graph_generation,
        RenderQueueClass::Graphics,
        passes,
        leases,
    )
    .map_err(|error| GraphicsError::WgpuValidation(error.to_string()))
}

fn rhi_resource_id(
    resource: RenderGraphResource,
    graph_generation: u64,
) -> Result<RhiGraphResourceId, GraphicsError> {
    let (kind, index, generation) = match resource {
        RenderGraphResource::TransientTexture(handle) => (
            RhiGraphResourceKind::Texture,
            handle.index(),
            handle.generation(),
        ),
        RenderGraphResource::TransientBuffer(handle) => (
            RhiGraphResourceKind::Buffer,
            handle.index(),
            handle.generation(),
        ),
        RenderGraphResource::External(handle) => (
            RhiGraphResourceKind::External,
            handle.index(),
            handle.generation(),
        ),
    };
    if generation != graph_generation {
        return Err(GraphicsError::WgpuValidation(format!(
            "graph resource generation {generation} does not match graph generation {graph_generation}"
        )));
    }
    Ok(RhiGraphResourceId::new(kind, index, graph_generation))
}

fn rhi_access_id(access: crate::render_graph::RenderGraphResourceAccessId) -> RhiGraphAccessId {
    RhiGraphAccessId::new(
        access.pass().index(),
        access.pass().generation(),
        access.access_index(),
    )
}

fn rhi_queue_lane(queue: QueueLane) -> RhiGraphQueueLane {
    match queue {
        QueueLane::Graphics => RhiGraphQueueLane::Graphics,
        QueueLane::AsyncCompute => RhiGraphQueueLane::AsyncCompute,
        QueueLane::AsyncCopy => RhiGraphQueueLane::AsyncCopy,
    }
}

fn rhi_access_kind(access: RenderGraphResourceAccessKind) -> RhiGraphResourceAccessKind {
    match access {
        RenderGraphResourceAccessKind::Read => RhiGraphResourceAccessKind::Read,
        RenderGraphResourceAccessKind::Write => RhiGraphResourceAccessKind::Write,
    }
}

fn rhi_resource_state(state: GraphResourceState) -> RhiResourceState {
    match state {
        GraphResourceState::Legacy => RhiResourceState::Legacy,
        GraphResourceState::SampledTexture => RhiResourceState::SampledTexture,
        GraphResourceState::StorageTextureRead => RhiResourceState::StorageTextureRead,
        GraphResourceState::StorageTextureWrite => RhiResourceState::StorageTextureWrite,
        GraphResourceState::ColorAttachment => RhiResourceState::ColorAttachment,
        GraphResourceState::DepthStencilAttachment => RhiResourceState::DepthStencilAttachment,
        GraphResourceState::UniformBuffer => RhiResourceState::UniformBuffer,
        GraphResourceState::StorageBufferRead => RhiResourceState::StorageBufferRead,
        GraphResourceState::StorageBufferReadWrite => RhiResourceState::StorageBufferReadWrite,
        GraphResourceState::CopySource => RhiResourceState::CopySource,
        GraphResourceState::CopyDestination => RhiResourceState::CopyDestination,
        GraphResourceState::Indirect => RhiResourceState::Indirect,
        GraphResourceState::Present => RhiResourceState::Present,
        GraphResourceState::Readback => RhiResourceState::Readback,
    }
}

fn rhi_access_range(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
    range: RenderGraphResourceAccessRange,
) -> Result<RhiGraphAccessRange, GraphicsError> {
    match range {
        RenderGraphResourceAccessRange::UnresolvedExternal => {
            if declaration.kind != crate::render_graph::RenderGraphResourceKind::External {
                return Err(GraphicsError::WgpuValidation(format!(
                    "unresolved graph access `{}` is not an external resource",
                    declaration.name
                )));
            }
            Ok(RhiGraphAccessRange::UnresolvedExternal)
        }
        RenderGraphResourceAccessRange::Buffer(range) => {
            let total_size = physical_buffer_size(graph_resources, declaration)?;
            let size = range
                .size
                .unwrap_or_else(|| total_size.saturating_sub(range.offset));
            let end = range.offset.checked_add(size).ok_or_else(|| {
                GraphicsError::WgpuValidation(format!(
                    "graph buffer access `{}` range overflows",
                    declaration.name
                ))
            })?;
            if size == 0 || range.offset >= total_size || end > total_size {
                return Err(GraphicsError::WgpuValidation(format!(
                    "graph buffer access `{}` range [{}, {}) exceeds {} bytes",
                    declaration.name, range.offset, end, total_size
                )));
            }
            Ok(RhiGraphAccessRange::buffer(range.offset, size))
        }
        RenderGraphResourceAccessRange::Texture(range) => {
            let descriptor = physical_texture_desc_for_bounds(graph_resources, declaration)?;
            let mip_end = texture_range_end(
                range.base_mip_level,
                range.mip_level_count,
                descriptor.mip_levels,
                "mip",
                &declaration.name,
            )?;
            let layer_end = texture_range_end(
                range.base_array_layer,
                range.array_layer_count,
                descriptor.array_layer_count(),
                "array layer",
                &declaration.name,
            )?;
            let supported_aspects = texture_aspect_mask(&descriptor);
            let aspect_mask =
                texture_access_aspect_mask(range.aspect, supported_aspects, &declaration.name)?;
            Ok(RhiGraphAccessRange::texture(
                range.base_mip_level,
                mip_end,
                range.base_array_layer,
                layer_end,
                aspect_mask,
            ))
        }
    }
}

fn texture_range_end(
    start: u32,
    count: Option<u32>,
    bound: u32,
    label: &str,
    resource_name: &str,
) -> Result<u32, GraphicsError> {
    let count = count.unwrap_or_else(|| bound.saturating_sub(start));
    let end = start.checked_add(count).ok_or_else(|| {
        GraphicsError::WgpuValidation(format!(
            "graph texture `{resource_name}` {label} range overflows"
        ))
    })?;
    if count == 0 || start >= bound || end > bound {
        return Err(GraphicsError::WgpuValidation(format!(
            "graph texture `{resource_name}` {label} range [{start}, {end}) exceeds {bound}"
        )));
    }
    Ok(end)
}

fn texture_aspect_mask(desc: &crate::rhi::TextureDesc) -> u8 {
    if desc.format.is_depth() {
        if desc.format.has_stencil() {
            0x06
        } else {
            0x02
        }
    } else {
        0x01
    }
}

fn texture_access_aspect_mask(
    aspect: RenderGraphTextureAspect,
    supported_aspects: u8,
    resource_name: &str,
) -> Result<u8, GraphicsError> {
    let requested = match aspect {
        RenderGraphTextureAspect::All => supported_aspects,
        RenderGraphTextureAspect::Color => 0x01,
        RenderGraphTextureAspect::Depth => 0x02,
        RenderGraphTextureAspect::Stencil => 0x04,
    };
    if requested == 0 || requested & !supported_aspects != 0 {
        return Err(GraphicsError::WgpuValidation(format!(
            "graph texture `{resource_name}` requests an unsupported aspect"
        )));
    }
    Ok(requested)
}

fn declaration_texture_desc(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
) -> Result<crate::rhi::TextureDesc, GraphicsError> {
    if let Some(desc) = graph_resources.physical_texture_desc(&declaration.name) {
        return Ok(desc.clone());
    }
    match &declaration.desc {
        RenderGraphResourceDesc::Texture(desc) => Ok(desc.clone()),
        RenderGraphResourceDesc::External => declaration
            .external_texture_desc
            .clone()
            .or_else(|| {
                graph_resources
                    .physical_texture_desc(&declaration.name)
                    .cloned()
            })
            .ok_or_else(|| {
                GraphicsError::WgpuValidation(format!(
                    "graph texture `{}` has no physical descriptor",
                    declaration.name
                ))
            }),
        RenderGraphResourceDesc::Buffer(_) => Err(GraphicsError::WgpuValidation(format!(
            "graph resource `{}` is declared as a buffer",
            declaration.name
        ))),
    }
}

fn declaration_buffer_size(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
) -> Result<u64, GraphicsError> {
    let size = match &declaration.desc {
        RenderGraphResourceDesc::Buffer(desc) => Some(desc.size_bytes),
        RenderGraphResourceDesc::External => declaration
            .external_buffer_desc
            .as_ref()
            .map(|desc| desc.size_bytes)
            .or_else(|| graph_resources.physical_buffer_size(&declaration.name)),
        RenderGraphResourceDesc::Texture(_) => None,
    };
    let size = size.ok_or_else(|| {
        GraphicsError::WgpuValidation(format!(
            "graph buffer `{}` has no physical size",
            declaration.name
        ))
    })?;
    if size == 0 {
        return Err(GraphicsError::WgpuValidation(format!(
            "graph buffer `{}` has zero physical size",
            declaration.name
        )));
    }
    Ok(size)
}

fn physical_texture_desc_for_bounds(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
) -> Result<crate::rhi::TextureDesc, GraphicsError> {
    graph_resources
        .physical_texture_desc(&declaration.name)
        .cloned()
        .or_else(|| declaration_texture_desc(graph_resources, declaration).ok())
        .ok_or_else(|| {
            GraphicsError::WgpuValidation(format!(
                "graph texture `{}` has no physical descriptor",
                declaration.name
            ))
        })
}

fn physical_buffer_size(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
) -> Result<u64, GraphicsError> {
    graph_resources
        .physical_buffer_size(&declaration.name)
        .or_else(|| declaration_buffer_size(graph_resources, declaration).ok())
        .filter(|size| *size > 0)
        .ok_or_else(|| {
            GraphicsError::WgpuValidation(format!(
                "graph buffer `{}` has no non-zero physical size",
                declaration.name
            ))
        })
}

fn physical_lease(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
    access_id: RhiGraphAccessId,
    resource: RhiGraphResourceId,
    allocation_id: Option<u64>,
    range: RhiGraphAccessRange,
    device_id: DeviceId,
    generation: DeviceGeneration,
) -> Result<RhiGraphPhysicalResourceLease, GraphicsError> {
    let (bounds, physical): (RhiGraphResourceBounds, Arc<dyn std::any::Any + Send + Sync>) =
        match range {
            RhiGraphAccessRange::Buffer { .. } => {
                let buffer = graph_resources.buffer(&declaration.name).ok_or_else(|| {
                    GraphicsError::WgpuValidation(format!(
                        "graph buffer `{}` has no physical WGPU binding",
                        declaration.name
                    ))
                })?;
                let size = physical_buffer_size(graph_resources, declaration)?;
                let bounds = RhiGraphResourceBounds::buffer(size)
                    .map_err(|error| GraphicsError::WgpuValidation(error.to_owned()))?;
                (bounds, Arc::new(buffer.clone()))
            }
            RhiGraphAccessRange::Texture { .. } => {
                let desc = physical_texture_desc_for_bounds(graph_resources, declaration)?;
                let bounds = RhiGraphResourceBounds::texture(
                    desc.mip_levels,
                    desc.array_layer_count(),
                    texture_aspect_mask(&desc),
                )
                .map_err(|error| GraphicsError::WgpuValidation(error.to_owned()))?;
                if let Some(texture) = graph_resources.physical_texture(&declaration.name) {
                    (bounds, Arc::new(texture.clone()))
                } else if let Some(view) = graph_resources.texture_view(&declaration.name) {
                    (bounds, Arc::new(view.clone()))
                } else {
                    return Err(GraphicsError::WgpuValidation(format!(
                        "graph texture `{}` has no physical WGPU binding",
                        declaration.name
                    )));
                }
            }
            RhiGraphAccessRange::UnresolvedExternal => {
                return Err(GraphicsError::WgpuValidation(
                    "logical-only external access cannot request a physical lease".to_owned(),
                ));
            }
        };
    Ok(RhiGraphPhysicalResourceLease::new(
        access_id,
        resource,
        allocation_id,
        device_id,
        generation,
        bounds,
        physical,
    ))
}

pub(super) fn prepare_environment_ibl_runtime_cache_writeback(
    writebacks: &IblBakeRuntimeGraphWritebackQueue,
    backend: &RenderBackend,
    streamer: &ResourceStreamer,
    request: Option<IblBakeArtifactRequest>,
    reservation: Option<EnvironmentIblBakeReservation>,
    graph_resources: &RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<Option<PreparedIblBakeRuntimeGraphWriteback>, GraphicsError> {
    let Some(reservation) = reservation else {
        return Ok(None);
    };
    let Some(request) = request else {
        return Ok(None);
    };
    let Some(store) = streamer.asset_manager()?.ibl_bake_artifact_cache_store() else {
        return Ok(None);
    };
    writebacks
        .prepare(backend, store, request, reservation, graph_resources, graph)
        .map_err(|error| GraphicsError::Asset(error.to_string()))
}

#[cfg(test)]
fn attach_scene_velocity_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    graph_resources: &RenderGraphExecutionResources,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let resource_name = PostProcessGraphResourceNames::SCENE_VELOCITY;
    let Some(texture) = graph_resources.physical_texture(resource_name) else {
        return;
    };
    let Some(desc) = graph_resources.physical_texture_desc(resource_name) else {
        return;
    };
    if desc.format != TextureFormat::Rg16Float
        || desc.sample_count != 1
        || desc.depth != 1
        || desc.array_layers != 1
    {
        return;
    }
    let size = crate::core::math::UVec2::new(desc.width, desc.height);
    if size.x == 0 || size.y == 0 {
        return;
    }
    let Ok(bytes) = read_texture_rgba(device, queue, texture, size) else {
        return;
    };
    graph_execution_record.set_scene_velocity_readback_report(
        RenderSceneVelocityReadbackReport::from_raw_rg16_float_bytes(size, &bytes),
    );
    graph_execution_record.set_scene_velocity_readback_rg16_float_bytes(bytes);
}

#[cfg(test)]
fn attach_exposure_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    graph_resources: &RenderGraphExecutionResources,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) -> RenderExposureReadbackReport {
    let Some(buffer) = graph_resources.buffer(PostProcessGraphResourceNames::EXPOSURE_CURRENT)
    else {
        return RenderExposureReadbackReport::default();
    };
    let Ok(words) = read_buffer_f32x4(device, queue, buffer) else {
        return RenderExposureReadbackReport::default();
    };
    let report = RenderExposureReadbackReport::from_words(words);
    graph_execution_record.set_exposure_readback_report(report);
    report
}

#[cfg(test)]
fn attach_color_lut_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    graph_resources: &RenderGraphExecutionResources,
    exposure_readback_report: RenderExposureReadbackReport,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let resource_name = PostProcessGraphResourceNames::COLOR_LUT;
    let Some(texture) = graph_resources.owned_texture(resource_name) else {
        return;
    };
    let Some(desc) = graph_resources.owned_texture_desc(resource_name) else {
        return;
    };
    if desc.format != TextureFormat::Rgba16Float || desc.sample_count != 1 {
        return;
    }
    let size = [desc.width, desc.height, desc.depth];
    if size.iter().any(|extent| *extent == 0) {
        return;
    }
    let Ok(bytes) = read_texture_rgba16float_3d(device, queue, texture, size) else {
        return;
    };
    graph_execution_record.set_color_lut_readback_report(color_lut_readback_report_for_frame(
        streamer,
        frame,
        size,
        &bytes,
        exposure_readback_report,
    ));
}

#[cfg(test)]
fn color_lut_readback_report_for_frame(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    size: [u32; 3],
    bytes: &[u8],
    exposure_readback_report: RenderExposureReadbackReport,
) -> RenderColorLutReadbackReport {
    if let Some(reference) = UserColorLutReadbackReference::from_frame(streamer, frame, size) {
        return RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
            size,
            bytes,
            |source_color| reference.expected_rgb(source_color),
        );
    }
    if let Some(reference) =
        ColorTransformLutReadbackReference::from_frame(frame, exposure_readback_report)
    {
        return RenderColorLutReadbackReport::from_raw_rgba16_float_color_transform_bytes(
            size,
            bytes,
            |source_color| reference.expected_rgb(source_color),
        );
    }

    RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes(size, bytes)
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct ColorTransformLutReadbackReference {
    tonemap: RenderTonemapSettings,
    grading: RenderColorGradingSettings,
    exposure_multiplier: f32,
}

#[cfg(test)]
impl ColorTransformLutReadbackReference {
    fn from_frame(
        frame: &ViewportRenderFrame,
        exposure_readback_report: RenderExposureReadbackReport,
    ) -> Option<Self> {
        let effect_stack = frame.post_process().effect_stack;
        if effect_stack.color_lookup.is_enabled() || !exposure_readback_report.history_valid() {
            return None;
        }
        let grading = frame.post_process().color_grading;
        let exposure_multiplier = exposure_readback_report.multiplier();
        color_transform_requires_reference(grading, effect_stack, exposure_multiplier).then_some(
            Self {
                tonemap: effect_stack.tonemap,
                grading,
                exposure_multiplier,
            },
        )
    }

    fn expected_rgb(self, source_color: [f32; 3]) -> [f32; 3] {
        self.apply_color_grading(self.apply_tonemap(source_color))
    }

    fn apply_tonemap(self, color: [f32; 3]) -> [f32; 3] {
        let exposure = 2.0_f32.powf(self.tonemap.render_exposure_bias() as f32)
            * self.exposure_multiplier.max(0.0);
        let white_point = (self.tonemap.render_white_point() as f32).max(0.001);
        let mut mapped = map_color(color, |channel| (channel * exposure).max(0.0));
        mapped = match self.tonemap.operator {
            RenderTonemapOperator::None => mapped,
            RenderTonemapOperator::Reinhard => {
                map_color(mapped, |channel| channel / (1.0 + channel / white_point))
            }
            RenderTonemapOperator::Aces => map_color(mapped, |channel| {
                let a = 2.51;
                let b = 0.03;
                let c = 2.43;
                let d = 0.59;
                let e = 0.14;
                ((channel * (a * channel + b)) / (channel * (c * channel + d) + e)).clamp(0.0, 1.0)
            }),
            RenderTonemapOperator::Filmic => map_color(mapped, |channel| {
                let mapped = (channel - 0.004).max(0.0);
                (mapped * (6.2 * mapped + 0.5)) / (mapped * (6.2 * mapped + 1.7) + 0.06)
            }),
        };
        mapped
    }

    fn apply_color_grading(self, color: [f32; 3]) -> [f32; 3] {
        let exposure = (self.grading.exposure as f32).max(0.0);
        let contrast = (self.grading.contrast as f32).max(0.0);
        let saturation = (self.grading.saturation as f32).max(0.0);
        let gamma = (self.grading.gamma as f32).max(0.001);
        let tint = [
            (self.grading.tint.x as f32).max(0.0),
            (self.grading.tint.y as f32).max(0.0),
            (self.grading.tint.z as f32).max(0.0),
        ];
        let mut graded = map_color(color, |channel| channel * exposure);
        let luma = graded[0] * 0.2126 + graded[1] * 0.7152 + graded[2] * 0.0722;
        graded = [
            mix_channel(luma, graded[0], saturation),
            mix_channel(luma, graded[1], saturation),
            mix_channel(luma, graded[2], saturation),
        ];
        graded = map_color(graded, |channel| ((channel - 0.5) * contrast) + 0.5);
        graded = map_color(graded, |channel| channel.max(0.0));
        graded = map_color(graded, |channel| channel.powf(1.0 / gamma));
        [
            graded[0] * tint[0],
            graded[1] * tint[1],
            graded[2] * tint[2],
        ]
    }
}

#[cfg(test)]
fn color_transform_requires_reference(
    grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
    exposure_multiplier: f32,
) -> bool {
    grading != RenderColorGradingSettings::default()
        || effect_stack.tonemap.is_enabled()
        || (exposure_multiplier - 1.0).abs() > 0.0001
}

#[cfg(test)]
fn map_color(color: [f32; 3], mut map: impl FnMut(f32) -> f32) -> [f32; 3] {
    [map(color[0]), map(color[1]), map(color[2])]
}

#[cfg(test)]
struct UserColorLutReadbackReference {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    mode: UserColorLutReadbackMode,
    intensity: f32,
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum UserColorLutReadbackMode {
    Texture2d,
    Texture2dStrip { size: u32 },
    Texture3d { size: u32 },
}

#[cfg(test)]
impl UserColorLutReadbackMode {
    fn rgba_byte_len(self, width: u32, height: u32) -> Option<usize> {
        match self {
            Self::Texture2d | Self::Texture2dStrip { .. } => rgba8_len(width, height),
            Self::Texture3d { size } => (size as usize)
                .checked_mul(size as usize)?
                .checked_mul(size as usize)?
                .checked_mul(4),
        }
    }
}

#[cfg(test)]
impl UserColorLutReadbackReference {
    fn from_frame(
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        readback_size: [u32; 3],
    ) -> Option<Self> {
        let effect_stack = frame.post_process().effect_stack;
        if !user_lut_readback_supports_frame(frame.post_process().color_grading, effect_stack) {
            return None;
        }
        Self::from_settings(streamer, effect_stack.color_lookup, readback_size)
    }

    fn from_settings(
        streamer: &ResourceStreamer,
        settings: RenderColorLookupSettings,
        readback_size: [u32; 3],
    ) -> Option<Self> {
        let texture_id = settings
            .is_enabled()
            .then(|| settings.texture.map(|texture| texture.id()))
            .flatten()?;
        let texture = streamer
            .asset_manager()
            .ok()?
            .load_texture_asset(texture_id)
            .ok()?;
        let descriptor = texture.render_image_descriptor();
        let mode = user_lut_readback_mode(
            streamer,
            texture_id,
            settings.texture_layout,
            &descriptor,
            readback_size,
        )?;
        let TextureAsset {
            rgba,
            width,
            height,
            payload,
            ..
        } = texture;
        if payload != TexturePayload::Rgba8 || rgba.len() < mode.rgba_byte_len(width, height)? {
            return None;
        }

        Some(Self {
            rgba,
            width,
            height,
            mode,
            intensity: (settings.render_intensity() as f32).clamp(0.0, 1.0),
        })
    }

    fn expected_rgb(&self, source_color: [f32; 3]) -> [f32; 3] {
        let user_color = self.sample(source_color);
        [
            mix_channel(source_color[0], user_color[0], self.intensity),
            mix_channel(source_color[1], user_color[1], self.intensity),
            mix_channel(source_color[2], user_color[2], self.intensity),
        ]
    }

    fn sample(&self, color: [f32; 3]) -> [f32; 3] {
        match self.mode {
            UserColorLutReadbackMode::Texture2d => [
                self.sample_1d_channel(color[0]),
                self.sample_1d_channel(color[1]),
                self.sample_1d_channel(color[2]),
            ],
            UserColorLutReadbackMode::Texture2dStrip { size } => {
                let red = lut_axis_index(color[0], size);
                let green = lut_axis_index(color[1], size);
                let blue = lut_axis_index(color[2], size);
                let x = blue.saturating_mul(size).saturating_add(red);
                self.texel_rgb(x.min(self.width.saturating_sub(1)), green.min(size - 1))
            }
            UserColorLutReadbackMode::Texture3d { size } => {
                let red = lut_axis_index(color[0], size);
                let green = lut_axis_index(color[1], size);
                let blue = lut_axis_index(color[2], size);
                let x = red.min(self.width.saturating_sub(1));
                let y = green.min(self.height.saturating_sub(1));
                let z_offset = blue.saturating_mul(self.width.saturating_mul(self.height));
                self.texel_rgb_by_flat_index(z_offset.saturating_add(y * self.width + x))
            }
        }
    }

    fn sample_1d_channel(&self, value: f32) -> f32 {
        let x = lut_axis_index(value, self.width);
        self.texel_rgb(x.min(self.width.saturating_sub(1)), 0)[0]
    }

    fn texel_rgb(&self, x: u32, y: u32) -> [f32; 3] {
        self.texel_rgb_by_flat_index(y.saturating_mul(self.width).saturating_add(x))
    }

    fn texel_rgb_by_flat_index(&self, flat_index: u32) -> [f32; 3] {
        let offset = flat_index as usize * 4;
        if offset + 2 >= self.rgba.len() {
            return [0.0; 3];
        }
        [
            self.rgba[offset] as f32 / 255.0,
            self.rgba[offset + 1] as f32 / 255.0,
            self.rgba[offset + 2] as f32 / 255.0,
        ]
    }
}

#[cfg(test)]
fn user_lut_readback_supports_frame(
    color_grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> bool {
    color_grading == RenderColorGradingSettings::default()
        && effect_stack.tonemap == RenderTonemapSettings::default()
}

#[cfg(test)]
fn user_lut_readback_mode(
    streamer: &ResourceStreamer,
    texture_id: crate::core::resource::ResourceId,
    layout: RenderColorLookupTextureLayout,
    descriptor: &RenderImageDescriptor,
    readback_size: [u32; 3],
) -> Option<UserColorLutReadbackMode> {
    let lut_size = readback_size[0];
    if readback_size != [lut_size; 3] || lut_size == 0 {
        return None;
    }
    if streamer
        .prepared_post_process_lut_3d_view(texture_id, layout)
        .is_some()
        && layout.matches_texture_3d(descriptor)
        && descriptor.width == lut_size
        && descriptor.height == lut_size
        && descriptor.depth_or_array_layers == lut_size
    {
        return Some(UserColorLutReadbackMode::Texture3d { size: lut_size });
    }
    if let Some((_, is_strip)) = streamer.prepared_post_process_lut_2d_view(texture_id, layout) {
        if is_strip
            && layout.matches_texture_2d_strip(descriptor)
            && descriptor.width == lut_size.saturating_mul(lut_size)
            && descriptor.height == lut_size
        {
            return Some(UserColorLutReadbackMode::Texture2dStrip { size: lut_size });
        }
        if !is_strip && descriptor.width == lut_size && descriptor.height > 0 {
            return Some(UserColorLutReadbackMode::Texture2d);
        }
    }
    None
}

#[cfg(test)]
fn lut_axis_index(value: f32, size: u32) -> u32 {
    let max_index = size.max(1) - 1;
    (value.clamp(0.0, 1.0) * max_index as f32)
        .round()
        .min(max_index as f32) as u32
}

#[cfg(test)]
fn mix_channel(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
fn rgba8_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)
}

#[cfg(test)]
#[path = "submit_compiled_scene_frame/tests/cases.rs"]
mod submission_order_tests;
