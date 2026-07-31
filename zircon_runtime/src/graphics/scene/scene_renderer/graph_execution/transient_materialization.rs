use std::collections::BTreeMap;

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderGraphTransientAllocation, RenderGraphResourceDesc,
    RenderGraphResourceKind, RenderGraphResourceLifetime,
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
        if !should_materialize_texture_lifetime(resources, lifetimes, lifetime)? {
            continue;
        }
        slot_lifetimes
            .entry(TransientMaterializationSlotKey::from_allocation(allocation))
            .or_default()
            .push(lifetime);
    }

    for (slot_key, lifetimes) in slot_lifetimes {
        if let Some(desc) = compatible_texture_slot_desc(slot_key, &lifetimes)? {
            let backing_name = transient_texture_backing_name(slot_key);
            resources.insert_owned_texture_backing(
                backing_name.clone(),
                pool.acquire_texture(device, &desc),
                desc,
            );
            for lifetime in lifetimes {
                resources.bind_owned_texture_view(lifetime.name.clone(), &backing_name)?;
            }
        } else {
            for lifetime in lifetimes {
                let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
                    return Err(format!(
                        "render graph resource `{}` has mismatched lifetime kind and descriptor",
                        lifetime.name
                    ));
                };
                resources.insert_owned_texture(
                    lifetime.name.clone(),
                    pool.acquire_texture(device, desc),
                    desc.clone(),
                );
            }
        }
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
        resources.insert_buffer_backing(
            backing_name.clone(),
            pool.acquire_buffer(device, &desc),
            desc.clone(),
        );
        for lifetime in lifetimes {
            resources.bind_buffer(lifetime.name.clone(), &backing_name);
        }
    }
    Ok(())
}

fn should_materialize_texture_lifetime(
    resources: &RenderGraphExecutionResources,
    lifetimes: &[RenderGraphResourceLifetime],
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
    Ok(!desc.is_sparse_reserved()
        && ssr_pyramid_mip_alias_for_lifetimes(lifetimes, &lifetime.name).is_none())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransientMaterializationSlotKey {
    bucket_key_hash: u64,
    slot: usize,
}

impl TransientMaterializationSlotKey {
    fn from_allocation(allocation: &CompiledRenderGraphTransientAllocation) -> Self {
        Self {
            bucket_key_hash: allocation.bucket_key_hash,
            slot: allocation.slot,
        }
    }
}

fn transient_texture_backing_name(slot_key: TransientMaterializationSlotKey) -> String {
    format!(
        "rg-transient-texture-bucket-{:016x}-slot-{}",
        slot_key.bucket_key_hash, slot_key.slot
    )
}

fn transient_buffer_backing_name(slot_key: TransientMaterializationSlotKey) -> String {
    format!(
        "rg-transient-buffer-bucket-{:016x}-slot-{}",
        slot_key.bucket_key_hash, slot_key.slot
    )
}

fn compatible_texture_slot_desc(
    slot_key: TransientMaterializationSlotKey,
    lifetimes: &[&RenderGraphResourceLifetime],
) -> Result<Option<TextureDesc>, String> {
    let Some(first) = lifetimes.first() else {
        return Ok(None);
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
        if !texture_descs_can_share_wgpu_backing(&desc, next) {
            return Ok(None);
        }
        desc.usage |= next.usage;
    }

    Ok(Some(desc))
}

fn texture_descs_can_share_wgpu_backing(left: &TextureDesc, right: &TextureDesc) -> bool {
    left.width == right.width
        && left.height == right.height
        && left.depth == right.depth
        && left.mip_levels == right.mip_levels
        && left.sample_count == right.sample_count
        && left.format == right.format
        && left.dimension == right.dimension
        && left.residency == right.residency
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
            "render graph transient buffer bucket `{}` slot `{}` has no logical resources",
            slot_key.bucket_key_hash, slot_key.slot
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
        desc.size_bytes = desc.size_bytes.max(next.size_bytes);
        desc.usage |= next.usage;
    }

    Ok(desc)
}

pub(super) fn ssr_pyramid_mip_alias_for_lifetimes(
    lifetimes: &[RenderGraphResourceLifetime],
    name: &str,
) -> Option<(&'static str, u32)> {
    let (parent, mip_level) = ssr_pyramid_mip_alias(name)?;
    lifetimes
        .iter()
        .find(|lifetime| lifetime.name == parent)
        .and_then(|lifetime| match &lifetime.desc {
            RenderGraphResourceDesc::Texture(desc) if desc.mip_levels > mip_level => {
                Some((parent, mip_level))
            }
            _ => None,
        })
}

pub(super) fn ssr_pyramid_mip_alias(name: &str) -> Option<(&'static str, u32)> {
    match name {
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE => Some((
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
            1,
        )),
        _ => None,
    }
}
