use std::collections::HashMap;

use crate::render_graph::{
    RenderGraphBufferRange, RenderGraphComputeDispatchExtent, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessRange, RenderGraphResourceDeclaration, RenderGraphResourceDesc,
    RenderGraphVersionedAccessKey,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc};

use super::access_index::CompiledRenderGraphAccessIndex;
use super::CompiledRenderPass;
use crate::render_graph::error::RenderGraphError;

/// Immutable compiler-to-executor identity for a dynamic generic-compute dispatch.
///
/// WGPU resource resolution must start from this exact key. The workload's
/// logical label is an authoring input only and is not consulted at execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledRenderGraphComputeDispatchAccess {
    Indirect {
        access: RenderGraphVersionedAccessKey,
        offset: u64,
    },
    PerPixel {
        access: RenderGraphVersionedAccessKey,
        target_extent: [u32; 2],
        local_size: [u32; 2],
    },
}

/// Immutable dispatch-access packet for one live generic-compute pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphComputeDispatchAccessPacket {
    pub pass: crate::render_graph::RenderPassId,
    pub dispatch: CompiledRenderGraphComputeDispatchAccess,
}

const INDIRECT_DISPATCH_ARGUMENT_BYTES: u64 = 12;

pub(super) fn build_compute_dispatch_access_packets(
    passes: &[CompiledRenderPass],
    access_index: &CompiledRenderGraphAccessIndex,
    resource_declarations: &[RenderGraphResourceDeclaration],
) -> Result<
    HashMap<crate::render_graph::RenderPassId, CompiledRenderGraphComputeDispatchAccessPacket>,
    RenderGraphError,
> {
    let declarations_by_resource = resource_declarations
        .iter()
        .map(|declaration| (declaration.resource, declaration))
        .collect::<HashMap<_, _>>();
    let declarations_by_name = resource_declarations
        .iter()
        .map(|declaration| (declaration.name.as_str(), declaration))
        .collect::<HashMap<_, _>>();
    let mut packets = HashMap::new();

    for pass in passes {
        if pass.culled {
            continue;
        }
        let Some(workload) = &pass.compute_workload else {
            continue;
        };
        // Generic-compute metadata is the opt-in contract. Other executors may
        // carry a workload for scheduling or diagnostics without generic WGPU
        // resource binding.
        if pass.compute_pass_metadata.is_none() {
            continue;
        }
        let dispatch = match &workload.dispatch_extent {
            RenderGraphComputeDispatchExtent::FromBuffer { buffer, offset } => {
                let access = select_dispatch_access(
                    pass,
                    access_index,
                    &declarations_by_resource,
                    buffer,
                    DispatchResourceType::Buffer,
                    RenderGraphResourceAccessKind::Read,
                    "read buffer",
                )?;
                validate_indirect_dispatch_access(
                    pass,
                    buffer,
                    *offset,
                    access,
                    &declarations_by_resource,
                )?;
                Some(CompiledRenderGraphComputeDispatchAccess::Indirect {
                    access,
                    offset: *offset,
                })
            }
            RenderGraphComputeDispatchExtent::PerPixel { target, local_size } => {
                let access = select_per_pixel_dispatch_access(
                    pass,
                    access_index,
                    &declarations_by_resource,
                    target,
                )?;
                let target_extent =
                    per_pixel_target_extent(pass, target, access, &declarations_by_name)?;
                Some(CompiledRenderGraphComputeDispatchAccess::PerPixel {
                    access,
                    target_extent,
                    local_size: *local_size,
                })
            }
            _ => None,
        };
        if let Some(dispatch) = dispatch {
            packets.insert(
                pass.id,
                CompiledRenderGraphComputeDispatchAccessPacket {
                    pass: pass.id,
                    dispatch,
                },
            );
        }
    }

    Ok(packets)
}

#[derive(Clone, Copy)]
enum DispatchResourceType {
    Buffer,
    Texture,
}

fn select_per_pixel_dispatch_access(
    pass: &CompiledRenderPass,
    access_index: &CompiledRenderGraphAccessIndex,
    declarations_by_resource: &HashMap<
        crate::render_graph::RenderGraphResource,
        &RenderGraphResourceDeclaration,
    >,
    resource: &str,
) -> Result<RenderGraphVersionedAccessKey, RenderGraphError> {
    let write_candidates = dispatch_access_candidates(
        pass,
        access_index,
        resource,
        RenderGraphResourceAccessKind::Write,
    );
    if !write_candidates.is_empty() {
        let access =
            require_single_dispatch_access(pass, resource, "write texture", write_candidates)?;
        validate_dispatch_resource_type(
            pass,
            resource,
            access,
            declarations_by_resource,
            DispatchResourceType::Texture,
        )?;
        return Ok(access);
    }
    select_dispatch_access(
        pass,
        access_index,
        declarations_by_resource,
        resource,
        DispatchResourceType::Texture,
        RenderGraphResourceAccessKind::Read,
        "read texture",
    )
}

fn select_dispatch_access(
    pass: &CompiledRenderPass,
    access_index: &CompiledRenderGraphAccessIndex,
    declarations_by_resource: &HashMap<
        crate::render_graph::RenderGraphResource,
        &RenderGraphResourceDeclaration,
    >,
    resource: &str,
    expected_type: DispatchResourceType,
    access_kind: RenderGraphResourceAccessKind,
    required_access: &'static str,
) -> Result<RenderGraphVersionedAccessKey, RenderGraphError> {
    let access = require_single_dispatch_access(
        pass,
        resource,
        required_access,
        dispatch_access_candidates(pass, access_index, resource, access_kind),
    )?;
    validate_dispatch_resource_type(
        pass,
        resource,
        access,
        declarations_by_resource,
        expected_type,
    )?;
    Ok(access)
}

fn dispatch_access_candidates(
    pass: &CompiledRenderPass,
    access_index: &CompiledRenderGraphAccessIndex,
    resource: &str,
    access_kind: RenderGraphResourceAccessKind,
) -> Vec<RenderGraphVersionedAccessKey> {
    pass.resources
        .iter()
        .enumerate()
        .filter(|(_, access)| access.name == resource && access.access == access_kind)
        .filter_map(|(access_ordinal, _)| {
            let access_id = access_index.access_id_at(pass.id, access_ordinal)?;
            access_index.versioned_access_key(access_id)
        })
        .collect()
}

fn validate_dispatch_resource_type(
    pass: &CompiledRenderPass,
    resource: &str,
    access: RenderGraphVersionedAccessKey,
    declarations_by_resource: &HashMap<
        crate::render_graph::RenderGraphResource,
        &RenderGraphResourceDeclaration,
    >,
    expected_type: DispatchResourceType,
) -> Result<(), RenderGraphError> {
    let declaration = declarations_by_resource
        .get(&access.resource)
        .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
            resource: resource.to_owned(),
        })?;
    let matches_expected_type = match expected_type {
        DispatchResourceType::Buffer => buffer_desc(declaration).is_some(),
        DispatchResourceType::Texture => texture_desc(declaration).is_some(),
    };
    if matches_expected_type {
        return Ok(());
    }
    let required = match expected_type {
        DispatchResourceType::Buffer => "buffer",
        DispatchResourceType::Texture => "texture",
    };
    Err(
        RenderGraphError::ComputeDispatchResourcePhysicalContractMissing {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required,
        },
    )
}

fn require_single_dispatch_access(
    pass: &CompiledRenderPass,
    resource: &str,
    required_access: &'static str,
    candidates: Vec<RenderGraphVersionedAccessKey>,
) -> Result<RenderGraphVersionedAccessKey, RenderGraphError> {
    match candidates.as_slice() {
        [access] => Ok(*access),
        _ => Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required_access,
        }),
    }
}

fn validate_indirect_dispatch_access(
    pass: &CompiledRenderPass,
    resource: &str,
    offset: u64,
    access: RenderGraphVersionedAccessKey,
    declarations_by_resource: &HashMap<
        crate::render_graph::RenderGraphResource,
        &RenderGraphResourceDeclaration,
    >,
) -> Result<(), RenderGraphError> {
    let declaration = declarations_by_resource
        .get(&access.resource)
        .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
            resource: resource.to_owned(),
        })?;
    let descriptor = buffer_desc(declaration).ok_or_else(|| {
        RenderGraphError::ComputeDispatchResourcePhysicalContractMissing {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required: "buffer",
        }
    })?;
    if !descriptor.usage.contains(BufferUsage::INDIRECT) {
        return Err(RenderGraphError::ComputeIndirectDispatchUsageMissing {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required: BufferUsage::INDIRECT,
            actual: descriptor.usage,
        });
    }
    let RenderGraphResourceAccessRange::Buffer(range) = access.range else {
        return Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required_access: "read buffer",
        });
    };
    let (range_start, range_end) = resolved_buffer_range(range, descriptor);
    let command_end = offset
        .checked_add(INDIRECT_DISPATCH_ARGUMENT_BYTES)
        .ok_or_else(|| RenderGraphError::ComputeIndirectDispatchRangeOverflow {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            offset,
        })?;
    if range_start != offset || range_end != command_end {
        return Err(RenderGraphError::ComputeIndirectDispatchRangeNotExact {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            offset,
            range_start,
            range_end,
        });
    }
    Ok(())
}

fn per_pixel_target_extent(
    pass: &CompiledRenderPass,
    resource: &str,
    access: RenderGraphVersionedAccessKey,
    declarations_by_name: &HashMap<&str, &RenderGraphResourceDeclaration>,
) -> Result<[u32; 2], RenderGraphError> {
    let declaration = declarations_by_name.get(resource).ok_or_else(|| {
        RenderGraphError::ComputeDispatchResourceNotDeclared {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required_access: "read or write texture",
        }
    })?;
    let descriptor = texture_desc(declaration).ok_or_else(|| {
        RenderGraphError::ComputeDispatchResourcePhysicalContractMissing {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required: "texture",
        }
    })?;
    let RenderGraphResourceAccessRange::Texture(range) = access.range else {
        return Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
            pass: pass.name.clone(),
            resource: resource.to_owned(),
            required_access: "typed texture access",
        });
    };
    let target_base_mip_level = declaration
        .texture_view_alias
        .as_ref()
        .map_or(0, |alias| alias.range.base_mip_level);
    let target_mip_end = target_base_mip_level.saturating_add(descriptor.mip_levels);
    let Some(local_mip_level) = range
        .base_mip_level
        .checked_sub(target_base_mip_level)
        .filter(|mip_level| *mip_level < descriptor.mip_levels)
    else {
        return Err(
            RenderGraphError::ComputePerPixelDispatchAccessScopeOutsideTarget {
                pass: pass.name.clone(),
                resource: resource.to_owned(),
                selected_base_mip_level: range.base_mip_level,
                target_base_mip_level,
                target_mip_end,
            },
        );
    };
    Ok(texture_mip_extent(descriptor, local_mip_level))
}

fn buffer_desc(declaration: &RenderGraphResourceDeclaration) -> Option<&BufferDesc> {
    match &declaration.desc {
        RenderGraphResourceDesc::Buffer(desc) => Some(desc),
        RenderGraphResourceDesc::External => declaration.external_buffer_desc.as_ref(),
        RenderGraphResourceDesc::Texture(_) => None,
    }
}

fn texture_desc(declaration: &RenderGraphResourceDeclaration) -> Option<&TextureDesc> {
    match &declaration.desc {
        RenderGraphResourceDesc::Texture(desc) => Some(desc),
        RenderGraphResourceDesc::External => declaration.external_texture_desc.as_ref(),
        RenderGraphResourceDesc::Buffer(_) => None,
    }
}

fn resolved_buffer_range(range: RenderGraphBufferRange, descriptor: &BufferDesc) -> (u64, u64) {
    let end = range
        .size
        .and_then(|size| range.offset.checked_add(size))
        .unwrap_or(descriptor.size_bytes);
    (range.offset, end)
}

fn texture_mip_extent(texture: &TextureDesc, mip_level: u32) -> [u32; 2] {
    [
        texture.width.checked_shr(mip_level).unwrap_or(1).max(1),
        texture.height.checked_shr(mip_level).unwrap_or(1).max(1),
    ]
}
