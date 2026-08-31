use std::collections::HashMap;

use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange,
    RenderGraphResourceDeclaration, RenderGraphResourceKind, RenderGraphVersionedAccessKey,
};
use crate::rhi::{TextureDesc, TextureDimension};

use super::access_index::CompiledRenderGraphAccessIndex;
use super::CompiledRenderPass;
use crate::render_graph::error::RenderGraphError;

/// Immutable compiler-to-executor identity for one generic-compute binding slot.
///
/// A read/write storage binding deliberately carries two keys. A later executor
/// chooses its physical view or slice from the exact access ID rather than from
/// a resource name and access mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphComputeBindingAccess {
    pub binding: u32,
    pub kind: ComputeBindingKind,
    pub read_access: Option<RenderGraphVersionedAccessKey>,
    pub write_access: Option<RenderGraphVersionedAccessKey>,
}

/// Immutable binding packet for one live generic-compute pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphComputeBindingAccessPacket {
    pub pass: crate::render_graph::RenderPassId,
    bindings: Vec<CompiledRenderGraphComputeBindingAccess>,
}

impl CompiledRenderGraphComputeBindingAccessPacket {
    pub fn bindings(&self) -> &[CompiledRenderGraphComputeBindingAccess] {
        &self.bindings
    }

    pub fn binding(&self, binding: u32) -> Option<&CompiledRenderGraphComputeBindingAccess> {
        self.bindings
            .iter()
            .find(|candidate| candidate.binding == binding)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ResourceAccessLookupKey {
    resource: String,
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ScopedAccessLookupKey {
    resource_access: ResourceAccessLookupKey,
    metadata: RenderGraphResourceAccessMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ExternalAccessLookupKey {
    resource: String,
    access: RenderGraphResourceAccessKind,
}

struct PassComputeAccessLookup {
    scoped: HashMap<ScopedAccessLookupKey, Vec<RenderGraphVersionedAccessKey>>,
    resource_access: HashMap<ResourceAccessLookupKey, Vec<RenderGraphVersionedAccessKey>>,
    external: HashMap<ExternalAccessLookupKey, Vec<RenderGraphVersionedAccessKey>>,
}

pub(super) fn build_compute_binding_access_packets(
    passes: &[CompiledRenderPass],
    access_index: &CompiledRenderGraphAccessIndex,
    resource_declarations: &[RenderGraphResourceDeclaration],
) -> Result<
    HashMap<crate::render_graph::RenderPassId, CompiledRenderGraphComputeBindingAccessPacket>,
    RenderGraphError,
> {
    let declarations_by_name = resource_declarations
        .iter()
        .map(|declaration| (declaration.name.as_str(), declaration))
        .collect::<HashMap<_, _>>();
    let declarations_by_resource = resource_declarations
        .iter()
        .map(|declaration| (declaration.resource, declaration))
        .collect::<HashMap<_, _>>();
    let mut packets = HashMap::new();
    for pass in passes {
        let Some(metadata) = &pass.compute_pass_metadata else {
            continue;
        };
        if pass.culled {
            continue;
        }
        let lookup = PassComputeAccessLookup::new(pass, access_index)?;
        let bindings = metadata
            .bindings
            .iter()
            .map(|binding| {
                build_binding_access(
                    pass,
                    binding,
                    &lookup,
                    declarations_by_name.get(binding.resource.as_str()).copied(),
                    &declarations_by_resource,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        packets.insert(
            pass.id,
            CompiledRenderGraphComputeBindingAccessPacket {
                pass: pass.id,
                bindings,
            },
        );
    }
    Ok(packets)
}

impl PassComputeAccessLookup {
    fn new(
        pass: &CompiledRenderPass,
        access_index: &CompiledRenderGraphAccessIndex,
    ) -> Result<Self, RenderGraphError> {
        let mut scoped = HashMap::with_capacity(pass.resources.len());
        let mut resource_access = HashMap::with_capacity(pass.resources.len());
        let mut external = HashMap::new();
        for (access_ordinal, resource) in pass.resources.iter().enumerate() {
            let access_id = access_index
                .access_id_at(pass.id, access_ordinal)
                .ok_or_else(|| RenderGraphError::CompiledAccessIndexEntryMissing {
                    pass: pass.name.clone(),
                    access: access_ordinal,
                })?;
            let key = access_index
                .versioned_access_key(access_id)
                .ok_or_else(|| RenderGraphError::CompiledAccessIndexEntryMissing {
                    pass: pass.name.clone(),
                    access: access_ordinal,
                })?;
            let resource_key = ResourceAccessLookupKey {
                resource: resource.name.clone(),
                kind: resource.kind,
                access: resource.access,
            };
            resource_access
                .entry(resource_key.clone())
                .or_insert_with(Vec::new)
                .push(key);
            if resource.kind == RenderGraphResourceKind::External {
                external
                    .entry(ExternalAccessLookupKey {
                        resource: resource.name.clone(),
                        access: resource.access,
                    })
                    .or_insert_with(Vec::new)
                    .push(key);
                continue;
            }
            scoped
                .entry(ScopedAccessLookupKey {
                    resource_access: resource_key,
                    metadata: RenderGraphResourceAccessMetadata::new(key.range, key.intent),
                })
                .or_insert_with(Vec::new)
                .push(key);
        }
        Ok(Self {
            scoped,
            resource_access,
            external,
        })
    }
}

fn build_binding_access(
    pass: &CompiledRenderPass,
    binding: &BindingSchemaEntry,
    lookup: &PassComputeAccessLookup,
    declaration: Option<&RenderGraphResourceDeclaration>,
    declarations_by_resource: &HashMap<RenderGraphResource, &RenderGraphResourceDeclaration>,
) -> Result<CompiledRenderGraphComputeBindingAccess, RenderGraphError> {
    Ok(CompiledRenderGraphComputeBindingAccess {
        binding: binding.binding,
        kind: binding.kind,
        read_access: binding
            .compute_access_metadata(RenderGraphResourceAccessKind::Read)
            .map(|metadata| {
                resolve_binding_access(
                    pass,
                    binding,
                    RenderGraphResourceAccessKind::Read,
                    canonical_binding_access_metadata(
                        metadata,
                        declaration,
                        declarations_by_resource,
                    ),
                    lookup,
                )
            })
            .transpose()?,
        write_access: binding
            .compute_access_metadata(RenderGraphResourceAccessKind::Write)
            .map(|metadata| {
                resolve_binding_access(
                    pass,
                    binding,
                    RenderGraphResourceAccessKind::Write,
                    canonical_binding_access_metadata(
                        metadata,
                        declaration,
                        declarations_by_resource,
                    ),
                    lookup,
                )
            })
            .transpose()?,
    })
}

fn canonical_binding_access_metadata(
    metadata: RenderGraphResourceAccessMetadata,
    declaration: Option<&RenderGraphResourceDeclaration>,
    declarations_by_resource: &HashMap<RenderGraphResource, &RenderGraphResourceDeclaration>,
) -> RenderGraphResourceAccessMetadata {
    let Some(declaration) = declaration else {
        return metadata;
    };
    let range = match (&declaration.desc, metadata.range) {
        (
            crate::render_graph::RenderGraphResourceDesc::Buffer(desc),
            RenderGraphResourceAccessRange::Buffer(range),
        ) => {
            let size = range
                .size
                .or_else(|| desc.size_bytes.checked_sub(range.offset));
            RenderGraphResourceAccessRange::Buffer(
                crate::render_graph::RenderGraphBufferRange::new(range.offset, size),
            )
        }
        (
            crate::render_graph::RenderGraphResourceDesc::Texture(desc),
            RenderGraphResourceAccessRange::Texture(range),
        ) => canonical_texture_range(
            range,
            desc,
            declaration.texture_view_alias,
            declarations_by_resource,
        )
        .unwrap_or(metadata.range),
        _ => metadata.range,
    };
    RenderGraphResourceAccessMetadata::new(range, metadata.intent)
}

fn canonical_texture_range(
    range: crate::render_graph::RenderGraphTextureSubresourceRange,
    local_desc: &TextureDesc,
    alias: Option<crate::render_graph::RenderGraphTextureViewAlias>,
    declarations_by_resource: &HashMap<RenderGraphResource, &RenderGraphResourceDeclaration>,
) -> Option<RenderGraphResourceAccessRange> {
    let local_array_layers = texture_array_layer_count(local_desc);
    let local_mip_level_count = resolved_range_count(
        range.base_mip_level,
        range.mip_level_count,
        local_desc.mip_levels,
    )?;
    let local_array_layer_count = resolved_range_count(
        range.base_array_layer,
        range.array_layer_count,
        local_array_layers,
    )?;
    let (base_mip_level, base_array_layer, aspect) = match alias {
        Some(alias) => {
            let parent = declarations_by_resource
                .get(&RenderGraphResource::TransientTexture(alias.parent))?;
            let crate::render_graph::RenderGraphResourceDesc::Texture(parent_desc) = &parent.desc
            else {
                return None;
            };
            let parent_array_layers = texture_array_layer_count(parent_desc);
            let alias_mip_count = resolved_range_count(
                alias.range.base_mip_level,
                alias.range.mip_level_count,
                parent_desc.mip_levels,
            )?;
            let alias_array_count = resolved_range_count(
                alias.range.base_array_layer,
                alias.range.array_layer_count,
                parent_array_layers,
            )?;
            if range.base_mip_level.checked_add(local_mip_level_count)? > alias_mip_count
                || range
                    .base_array_layer
                    .checked_add(local_array_layer_count)?
                    > alias_array_count
            {
                return None;
            }
            (
                alias
                    .range
                    .base_mip_level
                    .checked_add(range.base_mip_level)?,
                alias
                    .range
                    .base_array_layer
                    .checked_add(range.base_array_layer)?,
                compose_texture_aspect(alias.range.aspect, range.aspect)?,
            )
        }
        None => (range.base_mip_level, range.base_array_layer, range.aspect),
    };
    Some(RenderGraphResourceAccessRange::Texture(
        crate::render_graph::RenderGraphTextureSubresourceRange {
            base_mip_level,
            mip_level_count: Some(local_mip_level_count),
            base_array_layer,
            array_layer_count: Some(local_array_layer_count),
            aspect,
        },
    ))
}

fn texture_array_layer_count(desc: &TextureDesc) -> u32 {
    match desc.dimension {
        TextureDimension::D2Array | TextureDimension::Cube => desc.depth,
        TextureDimension::D1 | TextureDimension::D2 | TextureDimension::D3 => 1,
    }
}

fn resolved_range_count(base: u32, count: Option<u32>, limit: u32) -> Option<u32> {
    let count = count.unwrap_or(limit.checked_sub(base)?);
    (count > 0 && base.checked_add(count).is_some_and(|end| end <= limit)).then_some(count)
}

fn compose_texture_aspect(
    parent_aspect: crate::render_graph::RenderGraphTextureAspect,
    local_aspect: crate::render_graph::RenderGraphTextureAspect,
) -> Option<crate::render_graph::RenderGraphTextureAspect> {
    match (parent_aspect, local_aspect) {
        (crate::render_graph::RenderGraphTextureAspect::All, aspect)
        | (aspect, crate::render_graph::RenderGraphTextureAspect::All) => Some(aspect),
        (left, right) if left == right => Some(left),
        _ => None,
    }
}

fn resolve_binding_access(
    pass: &CompiledRenderPass,
    binding: &BindingSchemaEntry,
    access: RenderGraphResourceAccessKind,
    expected_metadata: RenderGraphResourceAccessMetadata,
    lookup: &PassComputeAccessLookup,
) -> Result<RenderGraphVersionedAccessKey, RenderGraphError> {
    let kind = binding_resource_kind(binding.kind);
    let resource_access = ResourceAccessLookupKey {
        resource: binding.resource.clone(),
        kind,
        access,
    };
    let scoped = ScopedAccessLookupKey {
        resource_access: resource_access.clone(),
        metadata: expected_metadata,
    };
    if let Some(candidates) = lookup.scoped.get(&scoped) {
        return require_single_binding_access(pass, binding, access, candidates);
    }

    if let Some(candidates) = lookup.external.get(&ExternalAccessLookupKey {
        resource: binding.resource.clone(),
        access,
    }) {
        return require_single_binding_access(pass, binding, access, candidates);
    }

    if let Some(candidates) = lookup.resource_access.get(&resource_access) {
        let Some(key) = candidates.first() else {
            return Err(RenderGraphError::ComputeBindingAccessMissing {
                pass: pass.name.clone(),
                binding: binding.binding,
                resource: binding.resource.clone(),
                access,
            });
        };
        let actual = RenderGraphResourceAccessMetadata::new(key.range, key.intent);
        return Err(RenderGraphError::ComputeBindingAccessScopeMismatch {
            pass: pass.name.clone(),
            binding: binding.binding,
            resource: binding.resource.clone(),
            access,
            expected: expected_metadata,
            actual,
        });
    }

    Err(RenderGraphError::ComputeBindingAccessMissing {
        pass: pass.name.clone(),
        binding: binding.binding,
        resource: binding.resource.clone(),
        access,
    })
}

fn require_single_binding_access(
    pass: &CompiledRenderPass,
    binding: &BindingSchemaEntry,
    access: RenderGraphResourceAccessKind,
    candidates: &[RenderGraphVersionedAccessKey],
) -> Result<RenderGraphVersionedAccessKey, RenderGraphError> {
    match candidates {
        [access_key] => Ok(*access_key),
        candidates => Err(RenderGraphError::ComputeBindingAccessAmbiguous {
            pass: pass.name.clone(),
            binding: binding.binding,
            resource: binding.resource.clone(),
            access,
            candidate_count: candidates.len(),
        }),
    }
}

fn binding_resource_kind(binding: ComputeBindingKind) -> RenderGraphResourceKind {
    match binding {
        ComputeBindingKind::UniformBuffer
        | ComputeBindingKind::StorageBufferRead
        | ComputeBindingKind::StorageBufferReadWrite => RenderGraphResourceKind::TransientBuffer,
        ComputeBindingKind::SampledTexture | ComputeBindingKind::StorageTextureWrite => {
            RenderGraphResourceKind::TransientTexture
        }
    }
}
