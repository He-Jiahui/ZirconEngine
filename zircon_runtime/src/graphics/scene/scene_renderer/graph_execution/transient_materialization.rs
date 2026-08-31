use std::collections::BTreeMap;

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderGraphTransientAllocation,
    CompiledRenderGraphTransientAllocationId, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime,
};
use crate::rhi::{BufferDesc, TextureDesc};

use super::{
    TransientResourcePool, render_graph_execution_resources::RenderGraphExecutionResources,
};

pub(super) fn materialize_transient_texture_slots(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    pool: &mut TransientResourcePool,
) -> Result<(), String> {
    let lifetimes = graph.resource_lifetimes();
    let allocation_plan = graph.transient_allocation_plan();
    allocation_plan
        .validate_transient_allocation_intervals()
        .map_err(|error| format!("invalid transient texture allocation plan: {error}"))?;
    let mut slot_lifetimes =
        BTreeMap::<TransientMaterializationSlotKey, Vec<&RenderGraphResourceLifetime>>::new();

    for allocation in allocation_plan
        .allocations
        .iter()
        .filter(|allocation| allocation.kind == RenderGraphResourceKind::TransientTexture)
    {
        let Some(lifetime) = graph.resource_lifetime(allocation.resource) else {
            continue;
        };
        if !should_materialize_texture_lifetime(resources, lifetime)? {
            continue;
        }
        slot_lifetimes
            .entry(TransientMaterializationSlotKey::from_allocation(allocation))
            .or_default()
            .push(lifetime);
    }

    for (slot_key, lifetimes) in slot_lifetimes {
        let desc = compatible_texture_slot_desc(slot_key, &lifetimes)?;
        let backing_name = transient_texture_backing_name(slot_key);
        let allocation = pool
            .acquire_texture(device, &desc)
            .map_err(|error| error.to_string())?;
        resources.insert_owned_texture_backing(backing_name.clone(), allocation);
        for lifetime in lifetimes {
            resources.bind_owned_texture_view(lifetime.name.clone(), &backing_name)?;
        }
    }
    materialize_persistent_texture_lifetimes(resources, device, graph, pool)?;
    Ok(())
}

fn materialize_persistent_texture_lifetimes(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    pool: &mut TransientResourcePool,
) -> Result<(), String> {
    for lifetime in graph.resource_lifetimes().iter().filter(|lifetime| {
        lifetime.kind == RenderGraphResourceKind::TransientTexture
            && lifetime.usage.persistent
            && !lifetime.is_texture_view_alias()
    }) {
        if !should_materialize_texture_lifetime(resources, lifetime)? {
            continue;
        }
        let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
            return Err(format!(
                "render graph persistent texture `{}` has mismatched lifetime descriptor",
                lifetime.name
            ));
        };
        let backing_name = persistent_texture_backing_name(&lifetime.name);
        let allocation = pool
            .acquire_persistent_texture(device, desc)
            .map_err(|error| error.to_string())?;
        resources.insert_owned_texture_backing(backing_name.clone(), allocation);
        resources.bind_owned_texture_view(lifetime.name.clone(), &backing_name)?;
    }
    Ok(())
}

pub(super) fn materialize_transient_buffer_slots(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    pool: &mut TransientResourcePool,
) -> Result<(), String> {
    let allocation_plan = graph.transient_allocation_plan();
    allocation_plan
        .validate_transient_allocation_intervals()
        .map_err(|error| format!("invalid transient buffer allocation plan: {error}"))?;
    let mut slot_lifetimes =
        BTreeMap::<TransientMaterializationSlotKey, Vec<&RenderGraphResourceLifetime>>::new();

    for allocation in allocation_plan
        .allocations
        .iter()
        .filter(|allocation| allocation.kind == RenderGraphResourceKind::TransientBuffer)
    {
        let Some(lifetime) = graph.resource_lifetime(allocation.resource) else {
            continue;
        };
        if lifetime.imported || resources.has_buffer(&lifetime.name) {
            continue;
        }
        slot_lifetimes
            .entry(TransientMaterializationSlotKey::from_allocation(allocation))
            .or_default()
            .push(lifetime);
    }

    for (slot_key, lifetimes) in slot_lifetimes {
        let desc = buffer_slot_desc(slot_key, &lifetimes)?;
        let backing_name = transient_buffer_backing_name(slot_key);
        let allocation = pool
            .acquire_buffer(device, &desc)
            .map_err(|error| error.to_string())?;
        resources.insert_buffer_backing(backing_name.clone(), allocation);
        for lifetime in lifetimes {
            resources.bind_buffer(lifetime.name.clone(), &backing_name);
        }
    }
    Ok(())
}

fn should_materialize_texture_lifetime(
    resources: &RenderGraphExecutionResources,
    lifetime: &RenderGraphResourceLifetime,
) -> Result<bool, String> {
    if lifetime.imported {
        return Ok(false);
    }
    if resources.has_texture_view(&lifetime.name)
        && !preimported_transient_requires_owned_backing(&lifetime.name)
    {
        return Ok(false);
    }
    let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
        return Err(format!(
            "render graph resource `{}` has mismatched lifetime kind and descriptor",
            lifetime.name
        ));
    };
    Ok(!desc.is_sparse_reserved() && !lifetime.is_texture_view_alias())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransientMaterializationSlotKey {
    allocation_id: CompiledRenderGraphTransientAllocationId,
}

impl TransientMaterializationSlotKey {
    fn from_allocation(allocation: &CompiledRenderGraphTransientAllocation) -> Self {
        Self {
            allocation_id: allocation.allocation_id,
        }
    }
}

fn transient_texture_backing_name(slot_key: TransientMaterializationSlotKey) -> String {
    format!(
        "rg-transient-texture-allocation-{}",
        slot_key.allocation_id.index()
    )
}

fn persistent_texture_backing_name(logical_name: &str) -> String {
    format!("rg-persistent-texture-{logical_name}")
}

fn transient_buffer_backing_name(slot_key: TransientMaterializationSlotKey) -> String {
    format!(
        "rg-transient-buffer-allocation-{}",
        slot_key.allocation_id.index()
    )
}

fn compatible_texture_slot_desc(
    slot_key: TransientMaterializationSlotKey,
    lifetimes: &[&RenderGraphResourceLifetime],
) -> Result<TextureDesc, String> {
    let Some(first) = lifetimes.first() else {
        return Err(format!(
            "render graph transient texture allocation `{}` has no logical resources",
            slot_key.allocation_id.index()
        ));
    };
    let RenderGraphResourceDesc::Texture(first_desc) = &first.desc else {
        return Err(format!(
            "render graph resource `{}` has mismatched lifetime kind and descriptor",
            first.name
        ));
    };
    let mut desc = first_desc.clone();
    desc.label = Some(transient_texture_backing_name(slot_key));

    for lifetime in lifetimes.iter().skip(1) {
        let RenderGraphResourceDesc::Texture(next) = &lifetime.desc else {
            return Err(format!(
                "render graph resource `{}` has mismatched lifetime kind and descriptor",
                lifetime.name
            ));
        };
        if !texture_desc_matches_compiler_allocation(&desc, next) {
            return Err(format!(
                "render graph transient texture allocation `{}` contains incompatible resource `{}`; compiler allocation IDs must only join equal texture descriptors",
                slot_key.allocation_id.index(),
                lifetime.name
            ));
        }
    }

    Ok(desc)
}

fn texture_desc_matches_compiler_allocation(left: &TextureDesc, right: &TextureDesc) -> bool {
    left.width == right.width
        && left.height == right.height
        && left.depth == right.depth
        && left.mip_levels == right.mip_levels
        && left.sample_count == right.sample_count
        && left.format == right.format
        && super::transient_resource_pool::texture_view_format_bits(left)
            == super::transient_resource_pool::texture_view_format_bits(right)
        && left.dimension == right.dimension
        && left.residency == right.residency
        && left.usage == right.usage
}

fn preimported_transient_requires_owned_backing(name: &str) -> bool {
    name == PostProcessGraphResourceNames::FINAL_COMPOSITED
}

fn buffer_slot_desc(
    slot_key: TransientMaterializationSlotKey,
    lifetimes: &[&RenderGraphResourceLifetime],
) -> Result<BufferDesc, String> {
    let Some(first) = lifetimes.first() else {
        return Err(format!(
            "render graph transient buffer allocation `{}` has no logical resources",
            slot_key.allocation_id.index()
        ));
    };
    let RenderGraphResourceDesc::Buffer(first_desc) = &first.desc else {
        return Err(format!(
            "render graph resource `{}` has mismatched lifetime kind and descriptor",
            first.name
        ));
    };
    let mut desc = BufferDesc::new(
        transient_buffer_backing_name(slot_key),
        first_desc.size_bytes,
        first_desc.usage,
    );

    for lifetime in lifetimes.iter().skip(1) {
        let RenderGraphResourceDesc::Buffer(next) = &lifetime.desc else {
            return Err(format!(
                "render graph resource `{}` has mismatched lifetime kind and descriptor",
                lifetime.name
            ));
        };
        if desc.size_bytes != next.size_bytes || desc.usage != next.usage {
            return Err(format!(
                "render graph transient buffer allocation `{}` contains incompatible resource `{}`; compiler allocation IDs must only join equal buffer descriptors",
                slot_key.allocation_id.index(),
                lifetime.name
            ));
        }
    }

    Ok(desc)
}
