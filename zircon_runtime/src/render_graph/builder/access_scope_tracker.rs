use std::collections::{BTreeMap, HashMap, HashSet};

use crate::rhi::TextureDesc;

use super::super::access::{
    RenderGraphBufferRange, RenderGraphResourceAccessId, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange, RenderGraphTextureAspect,
    RenderGraphTextureSubresourceRange,
};
use super::super::error::RenderGraphError;
use super::super::types::{
    QueueLane, RenderGraphAttachmentStoreOp, RenderGraphResource, RenderGraphResourceDesc,
    RenderGraphResourceVersionToken, RenderGraphTextureViewAlias, RenderPassId,
};
use super::ResourceNode;

mod buffer_scope_history;
#[cfg(test)]
pub(super) use buffer_scope_history::with_whole_map_coalescing;
mod texture_scope;

use buffer_scope_history::{BufferScopeHistory, BufferSegment};
use texture_scope::{validate_texture_descriptor_for_tracking, TexturePlane, TextureScope};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct LatestWriter {
    pub(super) pass: RenderPassId,
    pub(super) access_index: usize,
    pub(super) store: RenderGraphAttachmentStoreOp,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ResourceAccessHistory {
    pub(super) latest_writer: Option<LatestWriter>,
    pub(super) latest_version_ordinal: u64,
    pub(super) readers_since_last_write: Vec<RenderPassId>,
    pub(super) latest_state: Option<LatestAccessState>,
}

impl ResourceAccessHistory {
    /// Pass accesses are processed together, so repeated reads in one pass need
    /// only one reader entry for later write-after-read ordering.
    pub(super) fn record_reader_pass(&mut self, pass: RenderPassId) {
        if self.readers_since_last_write.last().copied() != Some(pass) {
            self.readers_since_last_write.push(pass);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LatestAccessState {
    pub(super) access: RenderGraphResourceAccessId,
    pub(super) intent: RenderGraphResourceAccessIntent,
    pub(super) queue: QueueLane,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ScopedAccessState {
    pub(super) range: RenderGraphResourceAccessRange,
    pub(super) latest: Option<LatestAccessState>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AccessScopeWorkReceipt {
    pub(super) lookup_visits: usize,
    pub(super) split_visits: usize,
    pub(super) read_visits: usize,
    pub(super) update_visits: usize,
    pub(super) merge_visits: usize,
    /// Visits made by the state-plan state-read and history-update passes.
    /// Each visit is one mip/aspect key; array layers remain one interval.
    pub(super) plane_visits: usize,
}

impl AccessScopeWorkReceipt {
    pub(super) const fn delta_since(self, previous: Self) -> Self {
        Self {
            lookup_visits: self.lookup_visits - previous.lookup_visits,
            split_visits: self.split_visits - previous.split_visits,
            read_visits: self.read_visits - previous.read_visits,
            update_visits: self.update_visits - previous.update_visits,
            merge_visits: self.merge_visits - previous.merge_visits,
            plane_visits: self.plane_visits - previous.plane_visits,
        }
    }
}

#[derive(Clone, Debug)]
enum ScopeHistory {
    Texture(HashMap<TexturePlane, BufferScopeHistory>),
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
    Texture(TextureScope),
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
    Texture(HashMap<TexturePlane, BTreeMap<u64, BufferScopeOccupancy>>),
    Buffer(BTreeMap<u64, BufferScopeOccupancy>),
    Whole(usize),
}

/// Per-pass conflict index. Callers use one instance per pass and group by
/// logical identity plus direction, so it has no graph-global scheduling role.
///
/// Read/read overlap is intentionally allowed: two bindings may legally read
/// overlapping subresources (for example, a full mip-chain view and a coarse
/// mip alias) during one pass. Only overlapping writes need rejection because
/// there is no intra-pass ordering between them.
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
        if !is_write {
            return None;
        }
        let key = (scope.identity, is_write);
        match &scope.kind {
            PreparedScopeKind::Texture(texture_scope) => {
                let occupancy = self
                    .occupancy
                    .entry(key)
                    .or_insert_with(|| ScopeOccupancy::Texture(HashMap::new()));
                let ScopeOccupancy::Texture(occupied_cells) = occupancy else {
                    return Some(access_index);
                };
                for plane in texture_scope.planes() {
                    let intervals = occupied_cells.entry(plane).or_default();
                    if let Some((_, previous)) =
                        intervals.range(..=texture_scope.layer_start()).next_back()
                    {
                        if previous.end > texture_scope.layer_start() {
                            return Some(previous.access_index);
                        }
                    }
                    if let Some((&next_start, next)) =
                        intervals.range(texture_scope.layer_start()..).next()
                    {
                        if texture_scope.layer_end() > next_start {
                            return Some(next.access_index);
                        }
                    }
                }
                for plane in texture_scope.planes() {
                    occupied_cells.entry(plane).or_default().insert(
                        texture_scope.layer_start(),
                        BufferScopeOccupancy {
                            end: texture_scope.layer_end(),
                            access_index,
                        },
                    );
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
/// actually touches. Texture work scales with selected mip/aspect planes and
/// layer intervals; buffer work scales with touched intervals rather than bytes.
pub(super) struct AccessScopeTracker {
    descriptors: HashMap<RenderGraphResource, ScopeDescriptor>,
    texture_view_aliases: HashMap<RenderGraphResource, RenderGraphTextureViewAlias>,
    histories: HashMap<usize, ScopeHistory>,
    next_version_ordinals: HashMap<usize, u64>,
    work: AccessScopeWorkReceipt,
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
            work: AccessScopeWorkReceipt::default(),
        }
    }

    pub(super) const fn work_receipt(&self) -> AccessScopeWorkReceipt {
        self.work
    }

    pub(super) fn prepare_scope(
        &mut self,
        identity: usize,
        resource: RenderGraphResource,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<PreparedAccessScope, RenderGraphError> {
        let descriptor = match self.descriptors.get(&resource).cloned() {
            Some(descriptor) => descriptor,
            None => {
                eprintln!(
                    "ZR_TRACE scope-descriptor-missing resource={resource:?} identity={identity} metadata={metadata:?}"
                );
                return Err(RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                });
            }
        };
        // Keep tracker construction device-neutral, but reject malformed
        // texture shapes before canonicalization can expand any subresource
        // range or allocate history for the descriptor.
        if let ScopeDescriptor::Texture(desc) = &descriptor {
            validate_texture_descriptor_for_tracking(resource, desc)?;
        }
        let metadata = self.project_texture_view_alias_scope(resource, metadata)?;
        let metadata = Self::canonicalize_access_metadata(resource, &descriptor, metadata)?;
        let precise = !matches!(
            metadata.intent,
            super::super::access::RenderGraphResourceAccessIntent::Legacy
        );
        let kind = match (&descriptor, metadata.range) {
            (ScopeDescriptor::Texture(desc), RenderGraphResourceAccessRange::Texture(range)) => {
                PreparedScopeKind::Texture(TextureScope::new(desc, range))
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
                eprintln!(
                    "ZR_TRACE scope-kind-mismatch resource={resource:?} descriptor={descriptor:?} metadata={metadata:?}"
                );
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
                    desc.array_layer_count(),
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
            // Legacy external accesses deliberately carry an unresolved range
            // at authoring time. Once an imported resource has a physical
            // descriptor, resolve that legacy whole-resource access to the
            // descriptor's complete texture scope so it participates in the
            // same subresource tracking as an explicitly typed access.
            (
                ScopeDescriptor::Texture(desc),
                RenderGraphResourceAccessRange::UnresolvedExternal,
            ) => RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
                base_mip_level: 0,
                mip_level_count: Some(desc.mip_levels),
                base_array_layer: 0,
                array_layer_count: Some(desc.array_layer_count()),
                aspect: RenderGraphTextureAspect::All,
            }),
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
            (
                ScopeDescriptor::Buffer { size_bytes: _ },
                RenderGraphResourceAccessRange::UnresolvedExternal,
            ) => RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::full()),
            (ScopeDescriptor::Whole, range) => range,
            _ => {
                eprintln!(
                    "ZR_TRACE scope-canonicalization-mismatch resource={resource:?} descriptor={descriptor:?} metadata={metadata:?}"
                );
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
            PreparedScopeKind::Texture(texture_scope) => {
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
                let mut result = Vec::new();
                for plane in texture_scope.planes() {
                    let history = histories
                        .entry(plane)
                        .or_insert_with(|| BufferScopeHistory::new(texture_scope.layer_limit()));
                    history.ensure_boundaries(
                        texture_scope.layer_start(),
                        texture_scope.layer_end(),
                        scope.identity,
                        &mut self.work,
                    )?;
                    let start_len = result.len();
                    result.extend(
                        history
                            .segments
                            .range(texture_scope.layer_start()..texture_scope.layer_end())
                            .map(|(_, segment)| segment.history.clone()),
                    );
                    self.work.read_visits += result.len() - start_len;
                }
                Ok(result)
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
                history.ensure_boundaries(*start, *end, scope.identity, &mut self.work)?;
                let histories = history
                    .segments
                    .range(*start..*end)
                    .map(|(_, segment)| segment.history.clone())
                    .collect::<Vec<_>>();
                self.work.read_visits += histories.len();
                Ok(histories)
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
                self.work.read_visits += 1;
                Ok(vec![history.clone()])
            }
        }
    }

    pub(super) fn current_states_for(
        &mut self,
        scope: &PreparedAccessScope,
    ) -> Result<Vec<ScopedAccessState>, RenderGraphError> {
        match &scope.kind {
            PreparedScopeKind::Texture(texture_scope) => {
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
                let mut states = Vec::new();
                for plane in texture_scope.planes() {
                    self.work.plane_visits += 1;
                    let history = histories
                        .entry(plane)
                        .or_insert_with(|| BufferScopeHistory::new(texture_scope.layer_limit()));
                    history.ensure_boundaries(
                        texture_scope.layer_start(),
                        texture_scope.layer_end(),
                        scope.identity,
                        &mut self.work,
                    )?;
                    let start_len = states.len();
                    states.extend(
                        history
                            .segments
                            .range(texture_scope.layer_start()..texture_scope.layer_end())
                            .map(|(layer_start, segment)| ScopedAccessState {
                                range: texture_layer_range(plane, *layer_start, segment.end),
                                latest: segment.history.latest_state,
                            }),
                    );
                    self.work.read_visits += states.len() - start_len;
                }
                Ok(states)
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
                history.ensure_boundaries(*start, *end, scope.identity, &mut self.work)?;
                let states = history
                    .segments
                    .range(*start..*end)
                    .map(|(segment_start, segment)| ScopedAccessState {
                        range: RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(
                            *segment_start,
                            Some(segment.end - *segment_start),
                        )),
                        latest: segment.history.latest_state,
                    })
                    .collect::<Vec<_>>();
                self.work.read_visits += states.len();
                Ok(states)
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
                self.work.read_visits += 1;
                Ok(vec![ScopedAccessState {
                    range: scope.metadata.range,
                    latest: history.latest_state,
                }])
            }
        }
    }

    pub(super) fn mutate_histories(
        &mut self,
        scope: &PreparedAccessScope,
        mut update: impl FnMut(&mut ResourceAccessHistory),
    ) -> Result<(), RenderGraphError> {
        match &scope.kind {
            PreparedScopeKind::Texture(texture_scope) => {
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
                for plane in texture_scope.planes() {
                    self.work.plane_visits += 1;
                    let history = histories
                        .entry(plane)
                        .or_insert_with(|| BufferScopeHistory::new(texture_scope.layer_limit()));
                    history.ensure_boundaries(
                        texture_scope.layer_start(),
                        texture_scope.layer_end(),
                        scope.identity,
                        &mut self.work,
                    )?;
                    let starts = history
                        .segments
                        .range(texture_scope.layer_start()..texture_scope.layer_end())
                        .map(|(segment_start, _)| *segment_start)
                        .collect::<Vec<_>>();
                    self.work.update_visits += starts.len();
                    for segment_start in starts {
                        let Some(segment) = history.segments.get_mut(&segment_start) else {
                            return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                                identity: scope.identity,
                            });
                        };
                        update(&mut segment.history);
                    }
                    history.merge_adjacent_equal_around(
                        texture_scope.layer_start(),
                        texture_scope.layer_end(),
                        &mut self.work,
                    );
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
                history.ensure_boundaries(*start, *end, scope.identity, &mut self.work)?;
                let starts = history
                    .segments
                    .range(*start..*end)
                    .map(|(segment_start, _)| *segment_start)
                    .collect::<Vec<_>>();
                self.work.update_visits += starts.len();
                for segment_start in starts {
                    let Some(segment) = history.segments.get_mut(&segment_start) else {
                        return Err(RenderGraphError::AccessScopeTrackerStateMismatch {
                            identity: scope.identity,
                        });
                    };
                    update(&mut segment.history);
                }
                history.merge_adjacent_equal_around(*start, *end, &mut self.work);
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
                self.work.update_visits += 1;
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
            PreparedScopeKind::Texture(texture_scope) => {
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
                for plane in texture_scope.planes() {
                    let Some(history) = histories.get(&plane) else {
                        continue;
                    };
                    writers.extend(
                        history
                            .overlapping_segments(
                                texture_scope.layer_start(),
                                texture_scope.layer_end(),
                            )
                            .filter_map(|(_, segment)| segment.history.latest_writer),
                    );
                }
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
                        .overlapping_segments(*start, *end)
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
    .ok_or_else(|| {
        if std::env::var_os("ZR_TRACE_RENDER_GRAPH").is_some() {
            eprintln!(
                "ZR_TRACE alias-range-failure=parent-mip alias={alias:?} local={local:?} parent_desc={parent:?}"
            );
        }
        RenderGraphError::ResourceDeclarationMissing {
            resource: format!("{:?}", alias.parent),
        }
    })?;
    let parent_array_layers = parent.array_layer_count();
    let alias_array_count = resolved_range_count(
        alias.range.base_array_layer,
        alias.range.array_layer_count,
        parent_array_layers,
    )
    .ok_or_else(|| {
        if std::env::var_os("ZR_TRACE_RENDER_GRAPH").is_some() {
            eprintln!(
                "ZR_TRACE alias-range-failure=parent-array alias={alias:?} local={local:?} parent_desc={parent:?}"
            );
        }
        RenderGraphError::ResourceDeclarationMissing {
            resource: format!("{:?}", alias.parent),
        }
    })?;
    let mip_level_count =
        resolved_range_count(local.base_mip_level, local.mip_level_count, alias_mip_count)
            .ok_or_else(|| {
                if std::env::var_os("ZR_TRACE_RENDER_GRAPH").is_some() {
                    eprintln!(
                        "ZR_TRACE alias-range-failure=local-mip alias={alias:?} local={local:?} parent_desc={parent:?} alias_mip_count={alias_mip_count}"
                    );
                }
                RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{:?}", alias.parent),
                }
            })?;
    let array_layer_count = resolved_range_count(
        local.base_array_layer,
        local.array_layer_count,
        alias_array_count,
    )
    .ok_or_else(|| {
        if std::env::var_os("ZR_TRACE_RENDER_GRAPH").is_some() {
            eprintln!(
                "ZR_TRACE alias-range-failure=local-array alias={alias:?} local={local:?} parent_desc={parent:?} alias_array_count={alias_array_count}"
            );
        }
        RenderGraphError::ResourceDeclarationMissing {
            resource: format!("{:?}", alias.parent),
        }
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

fn texture_layer_range(
    plane: TexturePlane,
    layer_start: u64,
    layer_end: u64,
) -> RenderGraphResourceAccessRange {
    RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
        base_mip_level: plane.mip_level,
        mip_level_count: Some(1),
        base_array_layer: layer_start as u32,
        array_layer_count: Some((layer_end - layer_start) as u32),
        aspect: plane.aspect,
    })
}

#[cfg(test)]
#[path = "tests/access_scope_tracker.rs"]
mod tests;

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
