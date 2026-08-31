use std::collections::{BTreeMap, HashMap, HashSet};

use crate::rhi::{TextureDesc, TextureDimension};

use super::super::access::{
    RenderGraphBufferRange, RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphTextureAspect, RenderGraphTextureSubresourceRange,
};
use super::super::error::RenderGraphError;
use super::super::types::{
    RenderGraphAttachmentStoreOp, RenderGraphResource, RenderGraphResourceDesc,
    RenderGraphResourceVersionToken, RenderGraphTextureViewAlias, RenderPassId,
};
use super::ResourceNode;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct LatestWriter {
    pub(super) pass: RenderPassId,
    pub(super) access_index: usize,
    pub(super) store: RenderGraphAttachmentStoreOp,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ResourceAccessHistory {
    pub(super) latest_writer: Option<LatestWriter>,
    pub(super) latest_version_ordinal: u64,
    pub(super) readers_since_last_write: Vec<RenderPassId>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct TextureCell {
    mip_level: u32,
    array_layer: u32,
    aspect: RenderGraphTextureAspect,
}

#[derive(Clone, Debug)]
struct BufferSegment {
    end: u64,
    history: ResourceAccessHistory,
}

#[derive(Clone, Debug, Default)]
struct BufferScopeHistory {
    segments: BTreeMap<u64, BufferSegment>,
}

#[derive(Clone, Debug)]
enum ScopeHistory {
    Texture(HashMap<TextureCell, ResourceAccessHistory>),
    Buffer(BufferScopeHistory),
    Whole(ResourceAccessHistory),
}

#[derive(Clone, Debug)]
enum ScopeDescriptor {
    Texture(TextureDesc),
    Buffer { size_bytes: u64 },
    Whole,
}

#[derive(Clone, Debug)]
enum PreparedScopeKind {
    Texture(Vec<TextureCell>),
    Buffer { start: u64, end: u64 },
    Whole,
}

/// A compilation-local canonicalized scope. It contains no RHI object and is
/// deliberately discarded once dependency inference has finished.
#[derive(Clone, Debug)]
pub(super) struct PreparedAccessScope {
    identity: usize,
    kind: PreparedScopeKind,
    precise: bool,
    metadata: RenderGraphResourceAccessMetadata,
}

impl PreparedAccessScope {
    pub(super) fn is_precise(&self) -> bool {
        self.precise
    }

    pub(super) const fn metadata(&self) -> RenderGraphResourceAccessMetadata {
        self.metadata
    }
}

#[derive(Clone, Copy, Debug)]
struct BufferScopeOccupancy {
    end: u64,
    access_index: usize,
}

#[derive(Debug)]
enum ScopeOccupancy {
    Texture(HashMap<TextureCell, usize>),
    Buffer(BTreeMap<u64, BufferScopeOccupancy>),
    Whole(usize),
}

/// Per-pass conflict index. Callers use one instance per pass and group by
/// logical identity plus direction, so it has no graph-global scheduling role.
#[derive(Default)]
pub(super) struct PassScopeConflictTracker {
    occupancy: HashMap<(usize, bool), ScopeOccupancy>,
}

impl PassScopeConflictTracker {
    /// Returns the earlier access that overlaps this same-direction scope.
    pub(super) fn register(
        &mut self,
        scope: &PreparedAccessScope,
        is_write: bool,
        access_index: usize,
    ) -> Option<usize> {
        let key = (scope.identity, is_write);
        match &scope.kind {
            PreparedScopeKind::Texture(cells) => {
                let occupancy = self.occupancy.entry(key).or_insert_with(|| {
                    ScopeOccupancy::Texture(HashMap::with_capacity(cells.len()))
                });
                let ScopeOccupancy::Texture(occupied_cells) = occupancy else {
                    return Some(access_index);
                };
                if let Some(previous_access) = cells
                    .iter()
                    .find_map(|cell| occupied_cells.get(cell).copied())
                {
                    return Some(previous_access);
                }
                for cell in cells {
                    occupied_cells.insert(*cell, access_index);
                }
                None
            }
            PreparedScopeKind::Buffer { start, end } => {
                let occupancy = self
                    .occupancy
                    .entry(key)
                    .or_insert_with(|| ScopeOccupancy::Buffer(BTreeMap::new()));
                let ScopeOccupancy::Buffer(intervals) = occupancy else {
                    return Some(access_index);
                };
                if let Some((_, previous)) = intervals.range(..=*start).next_back() {
                    if previous.end > *start {
                        return Some(previous.access_index);
                    }
                }
                if let Some((&next_start, next)) = intervals.range(*start..).next() {
                    if *end > next_start {
                        return Some(next.access_index);
                    }
                }
                intervals.insert(
                    *start,
                    BufferScopeOccupancy {
                        end: *end,
                        access_index,
                    },
                );
                None
            }
            PreparedScopeKind::Whole => match self.occupancy.entry(key) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(ScopeOccupancy::Whole(access_index));
                    None
                }
                std::collections::hash_map::Entry::Occupied(entry) => match entry.get() {
                    ScopeOccupancy::Whole(previous_access) => Some(*previous_access),
                    _ => Some(access_index),
                },
            },
        }
    }
}

/// Tracks only the portions of a logical transient resource that a graph
/// actually touches. Texture work scales with selected subresource cells;
/// buffer work scales with touched interval segments rather than byte count.
pub(super) struct AccessScopeTracker {
    descriptors: HashMap<RenderGraphResource, ScopeDescriptor>,
    texture_view_aliases: HashMap<RenderGraphResource, RenderGraphTextureViewAlias>,
    histories: HashMap<usize, ScopeHistory>,
    next_version_ordinals: HashMap<usize, u64>,
}

impl AccessScopeTracker {
    pub(super) fn new(resources: &[ResourceNode]) -> Self {
        let resource_nodes = resources
            .iter()
            .map(|resource| (resource.resource, resource))
            .collect::<HashMap<_, _>>();
        let descriptors = resources
            .iter()
            .map(|resource| {
                let source = resource
                    .texture_view_alias
                    .and_then(|alias| {
                        resource_nodes
                            .get(&RenderGraphResource::TransientTexture(alias.parent))
                            .copied()
                    })
                    .unwrap_or(resource);
                let descriptor = match &source.desc {
                    RenderGraphResourceDesc::Texture(desc) => {
                        ScopeDescriptor::Texture(desc.clone())
                    }
                    RenderGraphResourceDesc::Buffer(desc) => ScopeDescriptor::Buffer {
                        size_bytes: desc.size_bytes,
                    },
                    // A strong external descriptor is sufficient for compiler-only
                    // subresource tracking. P1-027 still owns the device-qualified
                    // physical lease and the eventual view/slice binding table.
                    RenderGraphResourceDesc::External => match (
                        source.external_texture_desc.as_ref(),
                        source.external_buffer_desc.as_ref(),
                    ) {
                        (Some(desc), _) => ScopeDescriptor::Texture(desc.clone()),
                        (None, Some(desc)) => ScopeDescriptor::Buffer {
                            size_bytes: desc.size_bytes,
                        },
                        (None, None) => ScopeDescriptor::Whole,
                    },
                };
                (resource.resource, descriptor)
            })
            .collect();
        let texture_view_aliases = resources
            .iter()
            .filter_map(|resource| {
                resource
                    .texture_view_alias
                    .map(|alias| (resource.resource, alias))
            })
            .collect();
        Self {
            descriptors,
            texture_view_aliases,
            histories: HashMap::new(),
            next_version_ordinals: HashMap::new(),
        }
    }

    pub(super) fn prepare_scope(
        &mut self,
        identity: usize,
        resource: RenderGraphResource,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<PreparedAccessScope, RenderGraphError> {
        let metadata = self.project_texture_view_alias_scope(resource, metadata)?;
        let descriptor = self.descriptors.get(&resource).cloned().ok_or_else(|| {
            RenderGraphError::ResourceDeclarationMissing {
                resource: format!("{resource:?}"),
            }
        })?;
        let metadata = Self::canonicalize_access_metadata(resource, &descriptor, metadata)?;
        let precise = !matches!(
            metadata.intent,
            super::super::access::RenderGraphResourceAccessIntent::Legacy
        );
        let kind = match (&descriptor, metadata.range) {
            (ScopeDescriptor::Texture(desc), RenderGraphResourceAccessRange::Texture(range)) => {
                PreparedScopeKind::Texture(texture_cells(desc, range))
            }
            (
                ScopeDescriptor::Buffer { size_bytes },
                RenderGraphResourceAccessRange::Buffer(range),
            ) => {
                let end = range.size.map_or(*size_bytes, |size| range.offset + size);
                PreparedScopeKind::Buffer {
                    start: range.offset,
                    end,
                }
            }
            (ScopeDescriptor::Whole, _) => PreparedScopeKind::Whole,
            // Access range validation has already returned a typed authoring
            // error before this compiler-only tracker is reached.
            _ => {
                return Err(RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                });
            }
        };
        self.ensure_history(identity, &descriptor);
        Ok(PreparedAccessScope {
            identity,
            kind,
            precise,
            metadata,
        })
    }

    fn project_texture_view_alias_scope(
        &self,
        resource: RenderGraphResource,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<RenderGraphResourceAccessMetadata, RenderGraphError> {
        let Some(alias) = self.texture_view_aliases.get(&resource).copied() else {
            return Ok(metadata);
        };
        let RenderGraphResourceAccessRange::Texture(local_range) = metadata.range else {
            return Err(RenderGraphError::ResourceDeclarationMissing {
                resource: format!("{resource:?}"),
            });
        };
        let parent_resource = RenderGraphResource::TransientTexture(alias.parent);
        let Some(ScopeDescriptor::Texture(parent_desc)) = self.descriptors.get(&parent_resource)
        else {
            return Err(RenderGraphError::ResourceDeclarationMissing {
                resource: format!("{parent_resource:?}"),
            });
        };
        let parent_range = project_texture_subresource_range(alias, local_range, parent_desc)?;
        Ok(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Texture(parent_range),
            metadata.intent,
        ))
    }

    fn canonicalize_access_metadata(
        resource: RenderGraphResource,
        descriptor: &ScopeDescriptor,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<RenderGraphResourceAccessMetadata, RenderGraphError> {
        let range = match (descriptor, metadata.range) {
            (ScopeDescriptor::Texture(desc), RenderGraphResourceAccessRange::Texture(range)) => {
                let mip_level_count = resolved_range_count(
                    range.base_mip_level,
                    range.mip_level_count,
                    desc.mip_levels,
                )
                .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                })?;
                let array_layer_count = resolved_range_count(
                    range.base_array_layer,
                    range.array_layer_count,
                    texture_array_layer_count(desc),
                )
                .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                })?;
                RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
                    base_mip_level: range.base_mip_level,
                    mip_level_count: Some(mip_level_count),
                    base_array_layer: range.base_array_layer,
                    array_layer_count: Some(array_layer_count),
                    aspect: range.aspect,
                })
            }
            (
                ScopeDescriptor::Buffer { size_bytes },
                RenderGraphResourceAccessRange::Buffer(range),
            ) => {
                let size = range
                    .size
                    .or_else(|| size_bytes.checked_sub(range.offset))
                    .filter(|size| *size > 0)
                    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                        resource: format!("{resource:?}"),
                    })?;
                RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(
                    range.offset,
                    Some(size),
                ))
            }
            (ScopeDescriptor::Whole, range) => range,
            _ => {
                return Err(RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                });
            }
        };
        Ok(RenderGraphResourceAccessMetadata::new(
            range,
            metadata.intent,
        ))
    }

    pub(super) fn histories_for(
        &mut self,
        scope: &PreparedAccessScope,
    ) -> Result<Vec<ResourceAccessHistory>, RenderGraphError> {
        match &scope.kind {
            PreparedScopeKind::Texture(cells) => {
                let Some(history) = self.histories.get_mut(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Texture(histories) = history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                Ok(cells
                    .iter()
                    .map(|cell| histories.entry(*cell).or_default().clone())
                    .collect())
            }
            PreparedScopeKind::Buffer { start, end } => {
                let Some(scope_history) = self.histories.get_mut(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Buffer(history) = scope_history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                history.ensure_boundaries(*start, *end, scope.identity)?;
                Ok(history
                    .segments
                    .range(*start..*end)
                    .map(|(_, segment)| segment.history.clone())
                    .collect())
            }
            PreparedScopeKind::Whole => {
                let Some(scope_history) = self.histories.get(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Whole(history) = scope_history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                Ok(vec![history.clone()])
            }
        }
    }

    pub(super) fn mutate_histories(
        &mut self,
        scope: &PreparedAccessScope,
        mut update: impl FnMut(&mut ResourceAccessHistory),
    ) -> Result<(), RenderGraphError> {
        match &scope.kind {
            PreparedScopeKind::Texture(cells) => {
                let Some(history) = self.histories.get_mut(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Texture(histories) = history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                for cell in cells {
                    update(histories.entry(*cell).or_default());
                }
                Ok(())
            }
            PreparedScopeKind::Buffer { start, end } => {
                let Some(scope_history) = self.histories.get_mut(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Buffer(history) = scope_history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                history.ensure_boundaries(*start, *end, scope.identity)?;
                let starts = history
                    .segments
                    .range(*start..*end)
                    .map(|(segment_start, _)| *segment_start)
                    .collect::<Vec<_>>();
                for segment_start in starts {
                    let Some(segment) = history.segments.get_mut(&segment_start) else {
                        return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                            identity: scope.identity,
                        });
                    };
                    update(&mut segment.history);
                }
                Ok(())
            }
            PreparedScopeKind::Whole => {
                let Some(scope_history) = self.histories.get_mut(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Whole(history) = scope_history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                update(history);
                Ok(())
            }
        }
    }

    pub(super) fn next_write_version(
        &mut self,
        identity: usize,
        resource: &str,
    ) -> Result<u64, RenderGraphError> {
        let ordinal = self.next_version_ordinals.entry(identity).or_insert(0);
        *ordinal =
            ordinal
                .checked_add(1)
                .ok_or_else(|| RenderGraphError::ResourceVersionExhausted {
                    resource: resource.to_owned(),
                })?;
        Ok(*ordinal)
    }

    /// Returns final writers only for the logical cull-root range.
    ///
    /// A texture view alias shares its parent's physical history, but its
    /// persistent or present role must not retain unrelated parent mips/layers.
    pub(super) fn cull_root_writers_for(
        &mut self,
        identity: usize,
        resource: RenderGraphResource,
    ) -> Result<Vec<LatestWriter>, RenderGraphError> {
        let descriptor = self.descriptors.get(&resource).ok_or_else(|| {
            RenderGraphError::ResourceDeclarationMissing {
                resource: format!("{resource:?}"),
            }
        })?;
        let root_range = match descriptor {
            ScopeDescriptor::Texture(_) => {
                RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full())
            }
            ScopeDescriptor::Buffer { .. } => {
                RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::full())
            }
            ScopeDescriptor::Whole => RenderGraphResourceAccessRange::UnresolvedExternal,
        };
        let scope = self.prepare_scope(
            identity,
            resource,
            RenderGraphResourceAccessMetadata::new(
                root_range,
                RenderGraphResourceAccessIntent::Legacy,
            ),
        )?;
        self.latest_writers_for_scope(&scope)
    }

    fn latest_writers_for_scope(
        &self,
        scope: &PreparedAccessScope,
    ) -> Result<Vec<LatestWriter>, RenderGraphError> {
        let mut writers = HashSet::new();
        match &scope.kind {
            PreparedScopeKind::Texture(cells) => {
                let Some(history) = self.histories.get(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Texture(histories) = history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                writers.extend(cells.iter().filter_map(|cell| {
                    histories
                        .get(cell)
                        .and_then(|history| history.latest_writer)
                }));
            }
            PreparedScopeKind::Buffer { start, end } => {
                let Some(scope_history) = self.histories.get(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Buffer(history) = scope_history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                writers.extend(
                    history
                        .segments
                        .range(*start..*end)
                        .filter_map(|(_, segment)| segment.history.latest_writer),
                );
            }
            PreparedScopeKind::Whole => {
                let Some(scope_history) = self.histories.get(&scope.identity) else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                let ScopeHistory::Whole(history) = scope_history else {
                    return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                        identity: scope.identity,
                    });
                };
                writers.extend(history.latest_writer);
            }
        }
        let mut writers = writers.into_iter().collect::<Vec<_>>();
        writers.sort_by_key(|writer| (writer.pass.index(), writer.access_index));
        Ok(writers)
    }

    fn ensure_history(&mut self, identity: usize, descriptor: &ScopeDescriptor) {
        self.histories
            .entry(identity)
            .or_insert_with(|| match descriptor {
                ScopeDescriptor::Texture(_) => ScopeHistory::Texture(HashMap::new()),
                ScopeDescriptor::Buffer { size_bytes } => {
                    let mut segments = BTreeMap::new();
                    segments.insert(
                        0,
                        BufferSegment {
                            end: *size_bytes,
                            history: ResourceAccessHistory::default(),
                        },
                    );
                    ScopeHistory::Buffer(BufferScopeHistory { segments })
                }
                ScopeDescriptor::Whole => ScopeHistory::Whole(ResourceAccessHistory::default()),
            });
    }
}

fn project_texture_subresource_range(
    alias: RenderGraphTextureViewAlias,
    local: RenderGraphTextureSubresourceRange,
    parent: &TextureDesc,
) -> Result<RenderGraphTextureSubresourceRange, RenderGraphError> {
    let alias_mip_count = resolved_range_count(
        alias.range.base_mip_level,
        alias.range.mip_level_count,
        parent.mip_levels,
    )
    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
        resource: format!("{:?}", alias.parent),
    })?;
    let parent_array_layers = texture_array_layer_count(parent);
    let alias_array_count = resolved_range_count(
        alias.range.base_array_layer,
        alias.range.array_layer_count,
        parent_array_layers,
    )
    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
        resource: format!("{:?}", alias.parent),
    })?;
    let mip_level_count =
        resolved_range_count(local.base_mip_level, local.mip_level_count, alias_mip_count)
            .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                resource: format!("{:?}", alias.parent),
            })?;
    let array_layer_count = resolved_range_count(
        local.base_array_layer,
        local.array_layer_count,
        alias_array_count,
    )
    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
        resource: format!("{:?}", alias.parent),
    })?;
    let aspect = compose_texture_aspect(alias.range.aspect, local.aspect).ok_or_else(|| {
        RenderGraphError::TextureViewAliasAspectUnsupported {
            alias: format!("{:?}", alias.parent),
            parent_name: format!("{:?}", alias.parent),
            aspect: local.aspect,
            format: parent.format,
        }
    })?;

    Ok(RenderGraphTextureSubresourceRange {
        base_mip_level: alias.range.base_mip_level + local.base_mip_level,
        mip_level_count: Some(mip_level_count),
        base_array_layer: alias.range.base_array_layer + local.base_array_layer,
        array_layer_count: Some(array_layer_count),
        aspect,
    })
}

fn resolved_range_count(base: u32, count: Option<u32>, limit: u32) -> Option<u32> {
    let count = count.unwrap_or(limit.checked_sub(base)?);
    (count > 0 && base.checked_add(count).is_some_and(|end| end <= limit)).then_some(count)
}

fn texture_array_layer_count(desc: &TextureDesc) -> u32 {
    match desc.dimension {
        TextureDimension::D2Array | TextureDimension::Cube => desc.depth,
        TextureDimension::D1 | TextureDimension::D2 | TextureDimension::D3 => 1,
    }
}

fn compose_texture_aspect(
    parent_view: RenderGraphTextureAspect,
    local_access: RenderGraphTextureAspect,
) -> Option<RenderGraphTextureAspect> {
    match (parent_view, local_access) {
        (RenderGraphTextureAspect::All, aspect) | (aspect, RenderGraphTextureAspect::All) => {
            Some(aspect)
        }
        (left, right) if left == right => Some(left),
        _ => None,
    }
}

impl BufferScopeHistory {
    fn ensure_boundaries(
        &mut self,
        start: u64,
        end: u64,
        identity: usize,
    ) -> Result<(), RenderGraphError> {
        self.split_at(start, identity)?;
        self.split_at(end, identity)
    }

    fn split_at(&mut self, boundary: u64, identity: usize) -> Result<(), RenderGraphError> {
        let Some((&start, segment)) = self.segments.range(..=boundary).next_back() else {
            return Err(RenderGraphError::AccessScopeTrackerStateMismatch { identity });
        };
        if boundary == start || boundary >= segment.end {
            return Ok(());
        }
        let end = segment.end;
        let history = segment.history.clone();
        let Some(segment) = self.segments.get_mut(&start) else {
            return Err(RenderGraphError::AccessScopeTrackerStateMismatch { identity });
        };
        segment.end = boundary;
        self.segments
            .insert(boundary, BufferSegment { end, history });
        Ok(())
    }
}

fn texture_cells(
    desc: &TextureDesc,
    range: crate::render_graph::RenderGraphTextureSubresourceRange,
) -> Vec<TextureCell> {
    let mip_end = range
        .mip_level_count
        .map_or(desc.mip_levels, |count| range.base_mip_level + count);
    let array_layers = texture_array_layer_count(desc);
    let array_end = range
        .array_layer_count
        .map_or(array_layers, |count| range.base_array_layer + count);
    let single_aspect = [range.aspect];
    let aspects: &[RenderGraphTextureAspect] = match range.aspect {
        RenderGraphTextureAspect::All if desc.format.has_stencil() => &[
            RenderGraphTextureAspect::Depth,
            RenderGraphTextureAspect::Stencil,
        ],
        RenderGraphTextureAspect::All if desc.format.is_depth() => {
            &[RenderGraphTextureAspect::Depth]
        }
        RenderGraphTextureAspect::All => &[RenderGraphTextureAspect::Color],
        RenderGraphTextureAspect::Color
        | RenderGraphTextureAspect::Depth
        | RenderGraphTextureAspect::Stencil => &single_aspect,
    };
    let mut cells = Vec::with_capacity(
        (mip_end - range.base_mip_level) as usize
            * (array_end - range.base_array_layer) as usize
            * aspects.len(),
    );
    for mip_level in range.base_mip_level..mip_end {
        for array_layer in range.base_array_layer..array_end {
            for aspect in aspects {
                cells.push(TextureCell {
                    mip_level,
                    array_layer,
                    aspect: *aspect,
                });
            }
        }
    }
    cells
}

pub(super) fn token_covers_scope(
    histories: &[ResourceAccessHistory],
    token: RenderGraphResourceVersionToken,
) -> bool {
    histories.iter().all(|history| {
        history.latest_writer.is_some_and(|writer| {
            writer.pass == token.producer_pass()
                && writer.access_index == token.producer_access_index()
        })
    })
}
