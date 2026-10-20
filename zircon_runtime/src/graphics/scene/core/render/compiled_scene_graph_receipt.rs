use std::sync::Arc;

use crate::graphics::backend::RenderBackend;
use crate::graphics::pipeline::CompiledRenderPipeline;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::types::GraphicsError;
use crate::render_graph::{
    QueueLane, RenderGraphResource, RenderGraphResourceAccessKind, RenderGraphResourceAccessRange,
    RenderGraphResourceDesc, RenderGraphResourceState as GraphResourceState,
    RenderGraphTextureAspect,
};
use crate::rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, RhiGraphAccessId, RhiGraphAccessRange,
    RhiGraphExecutionAccess, RhiGraphExecutionPass, RhiGraphExecutionReceipt,
    RhiGraphExecutionTransition, RhiGraphPhysicalResourceLease, RhiGraphQueueLane,
    RhiGraphResourceAccessKind, RhiGraphResourceBounds, RhiGraphResourceId, RhiGraphResourceKind,
    RhiGraphResourceState as RhiResourceState,
};

/// Lowers compiler-owned access rows into the device-qualified proof retained by the
/// submission timeline.  Every non-logical access must resolve to a concrete WGPU owner and
/// descriptor; a missing view, descriptor, range bound, epoch, or generation fails the frame
/// before a command buffer is admitted.
pub(super) fn build_graph_execution_receipt(
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
            let absence_bounds =
                optional_external_absence_bounds(graph_resources, declaration, binding.key, range)?;
            let has_physical_binding =
                range != RhiGraphAccessRange::UnresolvedExternal && absence_bounds.is_none();
            let mut access = RhiGraphExecutionAccess::new(
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
            if let Some(bounds) = absence_bounds {
                access = access
                    .with_optional_external_absence(bounds)
                    .map_err(|error| GraphicsError::WgpuValidation(error.to_string()))?;
            }
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

/// Absence is evidence about one materialized optional external access, not
/// permission to erase its typed range or to omit a transient/required lease.
pub(super) fn optional_external_absence_bounds(
    graph_resources: &RenderGraphExecutionResources,
    declaration: &crate::render_graph::RenderGraphResourceDeclaration,
    key: crate::render_graph::RenderGraphVersionedAccessKey,
    range: RhiGraphAccessRange,
) -> Result<Option<RhiGraphResourceBounds>, GraphicsError> {
    use crate::render_graph::{RenderGraphExternalResourceType, RenderGraphResourceKind};
    if declaration.kind != RenderGraphResourceKind::External
        || range == RhiGraphAccessRange::UnresolvedExternal
    {
        return Ok(None);
    }
    let (contract_key, contract) = graph_resources
        .external_access_contract_for_access(key.access_id)
        .ok_or_else(|| {
            GraphicsError::WgpuValidation(format!(
                "external access {:?} has no exact materialized contract",
                key.access_id
            ))
        })?;
    if contract_key != key || contract != declaration.external_binding {
        return Err(GraphicsError::WgpuValidation(format!(
            "external access {:?} does not match its materialized key and declaration contract",
            key.access_id
        )));
    }
    if contract.is_required() {
        return Ok(None);
    }
    let bounds = match (range, contract.resource_type) {
        (RhiGraphAccessRange::Texture { .. }, RenderGraphExternalResourceType::Texture) => {
            if graph_resources
                .optional_external_texture_view_for_access(key.access_id)
                .map_err(GraphicsError::WgpuValidation)?
                .is_some()
            {
                return Ok(None);
            }
            if graph_resources.texture_view(&declaration.name).is_some()
                || graph_resources
                    .physical_texture(&declaration.name)
                    .is_some()
            {
                return Err(GraphicsError::WgpuValidation(format!(
                    "external texture `{}` has physical backing without its exact access lease",
                    declaration.name
                )));
            }
            let desc = physical_texture_desc_for_bounds(graph_resources, declaration)?;
            RhiGraphResourceBounds::texture(
                desc.mip_levels,
                desc.array_layer_count(),
                texture_aspect_mask(&desc),
            )
        }
        (RhiGraphAccessRange::Buffer { .. }, RenderGraphExternalResourceType::Buffer) => {
            if graph_resources
                .optional_external_buffer_binding_for_access(key.access_id)
                .map_err(GraphicsError::WgpuValidation)?
                .is_some()
            {
                return Ok(None);
            }
            if graph_resources.buffer(&declaration.name).is_some() {
                return Err(GraphicsError::WgpuValidation(format!(
                    "external buffer `{}` has physical backing without its exact access lease",
                    declaration.name
                )));
            }
            RhiGraphResourceBounds::buffer(physical_buffer_size(graph_resources, declaration)?)
        }
        _ => {
            return Err(GraphicsError::WgpuValidation(format!(
                "external access {:?} typed range does not match its contract kind",
                key.access_id
            )))
        }
    }
    .map_err(|error| GraphicsError::WgpuValidation(error.to_owned()))?;
    Ok(Some(bounds))
}

pub(super) fn rhi_access_range(
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
